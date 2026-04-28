// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../../interfaces/IVestingVault.sol";
import "../../interfaces/ISanctionsOracle.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

// Minimal router interface for token liquidation (Uniswap-like)
interface IUniswapV2Router {
    function swapExactTokensForTokensSupportingFeeOnTransferTokens(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline
    ) external;
}

error TaxLiquidationFailed();

/**
 * @title VestingVault
 * @dev Vesting vault with real-time sanctions screening
 */
contract VestingVault is IVestingVault, Ownable, ReentrancyGuard {
    // ERC20 token being vested
    IERC20 public immutable token;
    
    // Sanctions oracle for compliance checks
    ISanctionsOracle public sanctionsOracle;
    
    // Mapping of beneficiary to grant
    mapping(address => Grant) public grants;
    
    // Array of all beneficiaries for enumeration
    address[] private _beneficiaries;
    
    // Mapping to check if beneficiary is in the array
    mapping(address => uint256) private _beneficiaryIndex;
    
    // Total amount of tokens in escrow (frozen due to sanctions)
    uint256 public totalEscrowedAmount;
    
    // Emergency pause flag
    bool public paused = false;

    // Address that must confirm tax configuration changes (second signer)
    address public taxAdmin;

    // DEX router for tax liquidations (can be zero if not used)
    address public dexRouter;

    struct TaxProposal {
        uint16 tax_bps;
        address tax_authority;
        address tax_asset;
        bool proposed;
    }

    mapping(address => TaxProposal) public pendingTaxProposals;
    
    /**
     * @dev Constructor
     * @param tokenAddress Address of the ERC20 token
     * @param sanctionsOracleAddress Address of the sanctions oracle
     * @param initialOwner Initial owner of the contract
     */
    constructor(
        address tokenAddress,
        address sanctionsOracleAddress,
        address initialOwner
    ) Ownable() {
        require(tokenAddress != address(0), "Invalid token address");
        require(sanctionsOracleAddress != address(0), "Invalid oracle address");
        
        token = IERC20(tokenAddress);
        sanctionsOracle = ISanctionsOracle(sanctionsOracleAddress);
        transferOwnership(initialOwner);
    }

    /**
     * @dev Owner proposes a tax configuration change for a specific grant.
     */
    function proposeTaxConfig(address beneficiary, uint16 tax_bps, address tax_authority, address tax_asset) external onlyOwner {
        require(grants[beneficiary].isActive, "No active grant");
        pendingTaxProposals[beneficiary] = TaxProposal({
            tax_bps: tax_bps,
            tax_authority: tax_authority,
            tax_asset: tax_asset,
            proposed: true
        });
    }

    /**
     * @dev Tax admin confirms a previously proposed tax configuration change, applying it atomically.
     */
    function confirmTaxConfig(address beneficiary) external {
        require(msg.sender == taxAdmin, "Only taxAdmin can confirm");
        TaxProposal memory p = pendingTaxProposals[beneficiary];
        require(p.proposed, "No pending proposal");

        Grant storage g = grants[beneficiary];
        g.tax_bps = p.tax_bps;
        g.tax_authority = p.tax_authority;
        g.tax_asset = p.tax_asset;

        delete pendingTaxProposals[beneficiary];
    }

    

    /**
     * @dev Set the tax admin address (owner only). Tax configuration changes require owner proposal and taxAdmin confirmation.
     */
    function setTaxAdmin(address _taxAdmin) external onlyOwner {
        require(_taxAdmin != address(0), "Invalid tax admin");
        taxAdmin = _taxAdmin;
    }

    /**
     * @dev Set the DEX router address used for token liquidation (owner only).
     */
    function setDexRouter(address _dexRouter) external onlyOwner {
        dexRouter = _dexRouter;
    }
    
    /**
     * @dev Creates a new vesting grant
     * @param beneficiary The beneficiary of the grant
     * @param amount Total amount of tokens to vest
     * @param start Start time of vesting (timestamp)
     * @param duration Duration of vesting in seconds
     */
    function createGrant(
        address beneficiary,
        uint256 amount,
        uint256 start,
        uint256 duration,
        uint16 tax_bps,
        address tax_authority,
        address tax_asset
    ) external onlyOwner {
        require(beneficiary != address(0), "Invalid beneficiary");
        require(amount > 0, "Amount must be positive");
        require(duration > 0, "Duration must be positive");
        require(!grants[beneficiary].isActive, "Grant already exists");
        
        uint256 s = start;
        if (s < block.timestamp) {
            s = block.timestamp;
        }

        grants[beneficiary] = Grant({
            amount: amount,
            start: s,
            duration: duration,
            claimed: 0,
            isActive: true,
            isEscrowed: false,
            escrowed_amount: 0,
            tax_bps: tax_bps,
            tax_authority: tax_authority,
            tax_asset: tax_asset,
            cumulative_taxes_paid: 0,
            tax_rounding_accumulator: 0
        });
        
        // Add to beneficiaries array
        _beneficiaries.push(beneficiary);
        _beneficiaryIndex[beneficiary] = _beneficiaries.length - 1;
        
        // Transfer tokens to this contract
        require(token.transferFrom(msg.sender, address(this), amount), "Transfer failed");
    }
    
    /**
     * @dev Claims vested tokens for a beneficiary
     * @param beneficiary The address claiming tokens
     */
    function claim(address beneficiary) external override nonReentrant {
        require(!paused, "Contract is paused");
        require(beneficiary != address(0), "Invalid beneficiary");
        require(grants[beneficiary].isActive, "No active grant");
        
        Grant storage grant = grants[beneficiary];
        uint256 claimable = _calculateClaimableAmount(grant);

        require(!grant.isEscrowed, "Tokens in escrow");

        // cap to remaining to avoid overflow from unexpected rounding
        uint256 remaining = grant.amount - grant.claimed;
        if (claimable > remaining) {
            claimable = remaining;
        }
        
        require(claimable > 0, "No tokens to claim");
        
        // Check sanctions before processing claim
        if (sanctionsOracle.isSanctioned(beneficiary)) {
            _freezeTokens(beneficiary, claimable);
            return;
        }
        
        // Process claim with tax withholding if configured
        uint256 gross = claimable;

        // Calculate tax using accumulator to avoid loss from rounding
        uint256 taxAmount = 0;
        if (grant.tax_bps > 0 && grant.tax_authority != address(0)) {
            uint256 raw = gross * uint256(grant.tax_bps);
            uint256 tax_floor = raw / 10000;
            uint256 remainder = raw % 10000;

            // accumulate remainder; when it reaches 10000, add 1 to tax
            uint256 acc = grant.tax_rounding_accumulator + remainder;
            if (acc >= 10000) {
                tax_floor += 1;
                acc -= 10000;
            }
            grant.tax_rounding_accumulator = acc;
            taxAmount = tax_floor;
        }

        uint256 net = gross - taxAmount;

        // Mark claimed (gross amount)
        grant.claimed += gross;

        // Perform transfers: first handle tax portion
        if (taxAmount > 0) {
            // If tax asset is same as vested token or not set, transfer directly
            if (grant.tax_asset == address(0) || grant.tax_asset == address(token)) {
                require(token.transfer(grant.tax_authority, taxAmount), "Tax transfer failed");
                grant.cumulative_taxes_paid += taxAmount;
            } else {
                // Need to liquidate taxAmount of `token` into `tax_asset` and send to authority
                if (dexRouter == address(0)) revert TaxLiquidationFailed();

                // Approve router
                require(token.approve(dexRouter, taxAmount), "Approve failed");

                address[] memory path = new address[](2);
                path[0] = address(token);
                path[1] = grant.tax_asset;

                // Try swap; revert with TaxLiquidationFailed on any failure
                try IUniswapV2Router(dexRouter).swapExactTokensForTokensSupportingFeeOnTransferTokens(
                    taxAmount,
                    1,
                    path,
                    grant.tax_authority,
                    block.timestamp + 300
                ) {
                    // We cannot read amounts from this function variant; assume success and increment by taxAmount as conservative record
                    grant.cumulative_taxes_paid += taxAmount;
                } catch {
                    revert TaxLiquidationFailed();
                }
            }
        }

        // Emit claim and tax events
        emit TokensClaimed(beneficiary, gross);
        emit TaxWithheld(beneficiary, gross, taxAmount, net);

        // Transfer net payout
        if (net > 0) {
            require(token.transfer(beneficiary, net), "Transfer failed");
        }
    }
    
    /**
     * @dev Gets the claimable amount for a beneficiary
     * @param beneficiary The address to check
     * @return The amount of tokens that can be claimed
     */
    function getClaimableAmount(address beneficiary) external view override returns (uint256) {
        Grant memory grant = grants[beneficiary];
        if (!grant.isActive || grant.isEscrowed) {
            return 0;
        }
        return _calculateClaimableAmount(grant);
    }
    
    /**
     * @dev Gets the grant details for a beneficiary
     * @param beneficiary The address to check
     * @return The grant details
     */
    function getGrant(address beneficiary) external view override returns (Grant memory) {
        return grants[beneficiary];
    }
    
    /**
     * @dev Updates the sanctions oracle address (owner only)
     * @param newOracle New sanctions oracle address
     */
    function updateSanctionsOracle(address newOracle) external onlyOwner {
        require(newOracle != address(0), "Invalid oracle address");
        sanctionsOracle = ISanctionsOracle(newOracle);
    }
    
    /**
     * @dev Releases tokens from escrow if beneficiary is no longer sanctioned
     * @param beneficiary The beneficiary whose tokens should be released
     */
    function releaseFromEscrow(address beneficiary) external nonReentrant {
        require(!paused, "Contract is paused");
        Grant storage grant = grants[beneficiary];
        require(grant.isEscrowed, "Tokens not in escrow");
        require(!sanctionsOracle.isSanctioned(beneficiary), "Beneficiary is still sanctioned");
        
        uint256 releasable = grant.escrowed_amount;
        grant.escrowed_amount = 0;
        grant.isEscrowed = false;
        // adjust global escrow
        if (totalEscrowedAmount >= releasable) {
            totalEscrowedAmount -= releasable;
        } else {
            totalEscrowedAmount = 0;
        }

        emit TokensReleased(beneficiary, releasable);

        require(token.transfer(beneficiary, releasable), "Transfer failed");
    }
    
    /**
     * @dev Emergency pause/unpause (owner only)
     * @param _paused New pause state
     */
    function setPaused(bool _paused) external onlyOwner {
        paused = _paused;
    }
    
    /**
     * @dev Gets all beneficiaries (paginated)
     * @param offset Starting index
     * @param limit Maximum number of beneficiaries to return
     * @return beneficiaries Array of beneficiary addresses
     */
    function getBeneficiaries(uint256 offset, uint256 limit) external view returns (address[] memory) {
        uint256 end = offset + limit;
        if (end > _beneficiaries.length) {
            end = _beneficiaries.length;
        }
        
        address[] memory beneficiaries = new address[](end - offset);
        for (uint256 i = offset; i < end; i++) {
            beneficiaries[i - offset] = _beneficiaries[i];
        }
        
        return beneficiaries;
    }
    
    /**
     * @dev Gets the total number of beneficiaries
     * @return count The number of beneficiaries
     */
    function getBeneficiaryCount() external view returns (uint256) {
        return _beneficiaries.length;
    }
    
    /**
     * @dev Calculates the claimable amount for a grant
     * @param grant The grant to calculate for
     * @return The claimable amount
     */
    function _calculateClaimableAmount(Grant memory grant) private view returns (uint256) {
        if (block.timestamp < grant.start) {
            return 0;
        }
        
        uint256 elapsed = block.timestamp - grant.start;
        uint256 vested = (grant.amount * elapsed) / grant.duration;
        
        if (vested > grant.amount) {
            vested = grant.amount;
        }
        
        return vested - grant.claimed;
    }
    
    /**
     * @dev Freezes tokens in escrow due to sanctions
     * @param beneficiary The sanctioned beneficiary
     * @param amount The amount to freeze
     */
    function _freezeTokens(address beneficiary, uint256 amount) private {
        Grant storage grant = grants[beneficiary];
        
        // Mark as escrowed
        grant.isEscrowed = true;
        grant.escrowed_amount += amount;
        totalEscrowedAmount += amount;
        
        emit TokensFrozen(beneficiary, amount);
    }
    
    /**
     * @dev Batch check sanctions for multiple beneficiaries
     * @param beneficiaries Array of beneficiary addresses
     * @return sanctioned Array of boolean results
     */
    function batchCheckSanctions(address[] calldata beneficiaries) external view returns (bool[] memory) {
        return sanctionsOracle.areSanctioned(beneficiaries);
    }
    
    /**
     * @dev Gets the contract's token balance
     * @return balance The token balance
     */
    function getTokenBalance() external view returns (uint256) {
        return token.balanceOf(address(this));
    }
    
    /**
     * @dev Gets the total amount of tokens available for claims (excluding escrow)
     * @return available The available amount
     */
    function getAvailableAmount() external view returns (uint256) {
        uint256 total = token.balanceOf(address(this));
        return total - totalEscrowedAmount;
    }
}
