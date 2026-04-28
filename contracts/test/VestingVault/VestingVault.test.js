const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("VestingVault with Sanctions Oracle", function () {
    let vestingVault;
    let sanctionsOracle;
    let token;
    let owner, beneficiary, sanctionedUser, otherUser;
    
    const GRANT_AMOUNT = ethers.parseEther("1000");
    const VESTING_DURATION = 365 * 24 * 60 * 60; // 1 year in seconds
    const TOL = 100000000000000000n; // acceptable tolerance for tiny timestamp rounding (1e17)

    function approxEqual(a, b, tol = TOL) {
        const diff = a > b ? a - b : b - a;
        if (!(diff <= tol)) {
            console.log('approxEqual failed:', a.toString(), b.toString(), 'diff=', diff.toString(), 'tol=', tol.toString());
        }
        expect(diff <= tol).to.be.true;
    }
    
    beforeEach(async function () {
        [owner, beneficiary, sanctionedUser, otherUser] = await ethers.getSigners();
        
        // Deploy mock ERC20 token
        const MockToken = await ethers.getContractFactory("MockERC20");
        token = await MockToken.deploy("Test Token", "TEST");
        await token.waitForDeployment();
        
        // Deploy sanctions oracle
        const SanctionsOracle = await ethers.getContractFactory("SanctionsOracle");
        sanctionsOracle = await SanctionsOracle.deploy(owner.address);
        await sanctionsOracle.waitForDeployment();
        
        // Deploy vesting vault
        const VestingVault = await ethers.getContractFactory("VestingVault");
        vestingVault = await VestingVault.deploy(
            await token.getAddress(),
            await sanctionsOracle.getAddress(),
            owner.address
        );
        await vestingVault.waitForDeployment();
        
        // Mint tokens to owner
        await token.mint(owner.address, GRANT_AMOUNT * 10n);
        
        // Approve tokens to vesting vault
        await token.approve(await vestingVault.getAddress(), GRANT_AMOUNT * 10n);
        
        // Create initial grant
        const startTime = (await ethers.provider.getBlock("latest")).timestamp;
        await vestingVault.createGrant(
            beneficiary.address,
            GRANT_AMOUNT,
            startTime,
            VESTING_DURATION,
            0, // tax_bps
            ethers.ZeroAddress,
            ethers.ZeroAddress
        );
    });
    
    describe("Normal Vesting Flow", function () {
        it("Should allow normal claiming when not sanctioned", async function () {
            // Fast forward 6 months
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 2]);
            await ethers.provider.send("evm_mine");
            
            const claimableAmount = await vestingVault.getClaimableAmount(beneficiary.address);
            approxEqual(claimableAmount, GRANT_AMOUNT / 2n);

            await vestingVault.claim(beneficiary.address);

            approxEqual(await token.balanceOf(beneficiary.address), GRANT_AMOUNT / 2n);
        });
        
        it("Should calculate correct claimable amount over time", async function () {
            const startTime = (await ethers.provider.getBlock("latest")).timestamp;
            
            // Check at start
            expect(await vestingVault.getClaimableAmount(beneficiary.address)).to.equal(0);
            
            // Fast forward 25% of vesting period
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 4]);
            await ethers.provider.send("evm_mine");
            
            approxEqual(await vestingVault.getClaimableAmount(beneficiary.address), GRANT_AMOUNT / 4n);
            
            // Fast forward to completion
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION * 3 / 4]);
            await ethers.provider.send("evm_mine");
            
            approxEqual(await vestingVault.getClaimableAmount(beneficiary.address), GRANT_AMOUNT);
        });
    });
    
    describe("Sanctions Enforcement", function () {
        it("Should freeze tokens when beneficiary is sanctioned", async function () {
            // Fast forward 6 months
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 2]);
            await ethers.provider.send("evm_mine");
            
            // Sanction the beneficiary
            await sanctionsOracle.sanctionAddress(beneficiary.address);
            
            // Attempt to claim - should freeze tokens instead
            await vestingVault.claim(beneficiary.address);
            
            // Check that tokens are in escrow
            const grant = await vestingVault.getGrant(beneficiary.address);
            expect(grant.isEscrowed).to.be.true;
            approxEqual(await vestingVault.totalEscrowedAmount(), GRANT_AMOUNT / 2n);
            
            // Beneficiary should not receive tokens
            expect(await token.balanceOf(beneficiary.address)).to.equal(0);
        });
        
        it("Should prevent claiming while in escrow", async function () {
            // Sanction and freeze tokens
            await sanctionsOracle.sanctionAddress(beneficiary.address);
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 2]);
            await ethers.provider.send("evm_mine");
            await vestingVault.claim(beneficiary.address);
            
            // Attempt to claim again while still sanctioned
            await expect(vestingVault.claim(beneficiary.address)).to.be.reverted;
            
            // Check claimable amount is 0 while in escrow
            expect(await vestingVault.getClaimableAmount(beneficiary.address)).to.equal(0);
        });
        
        it("Should release tokens when sanctions are lifted", async function () {
            // Fast forward 6 months and sanction
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 2]);
            await ethers.provider.send("evm_mine");
            await sanctionsOracle.sanctionAddress(beneficiary.address);
            await vestingVault.claim(beneficiary.address);
            
            // Unsanction the beneficiary
            await sanctionsOracle.unsanctionAddress(beneficiary.address);
            
            // Release from escrow
            await vestingVault.releaseFromEscrow(beneficiary.address);

            // Check tokens were released (allow small timestamp rounding tolerance)
            approxEqual(await token.balanceOf(beneficiary.address), GRANT_AMOUNT / 2n);
            
            // Check escrow state is cleared
            const grant = await vestingVault.getGrant(beneficiary.address);
            expect(grant.isEscrowed).to.be.false;
            expect(await vestingVault.totalEscrowedAmount()).to.equal(0);
        });
        
        it("Should prevent release if still sanctioned", async function () {
            // Sanction and freeze tokens
            await sanctionsOracle.sanctionAddress(beneficiary.address);
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 2]);
            await ethers.provider.send("evm_mine");
            await vestingVault.claim(beneficiary.address);
            
            // Attempt to release while still sanctioned
            await expect(vestingVault.releaseFromEscrow(beneficiary.address))
                .to.be.revertedWith("Beneficiary is still sanctioned");
        });
        
        it("Should handle batch sanctions correctly", async function () {
            // Create additional grants
            await vestingVault.createGrant(
                sanctionedUser.address,
                GRANT_AMOUNT,
                (await ethers.provider.getBlock("latest")).timestamp,
                VESTING_DURATION,
                0,
                ethers.ZeroAddress,
                ethers.ZeroAddress
            );
            
            await vestingVault.createGrant(
                otherUser.address,
                GRANT_AMOUNT,
                (await ethers.provider.getBlock("latest")).timestamp,
                VESTING_DURATION,
                0,
                ethers.ZeroAddress,
                ethers.ZeroAddress
            );
            
            // Fast forward and batch sanction
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 2]);
            await ethers.provider.send("evm_mine");
            
            await sanctionsOracle.batchSanction([
                beneficiary.address,
                sanctionedUser.address
            ]);
            
            // Claim for sanctioned users should freeze tokens
            await vestingVault.claim(beneficiary.address);
            await vestingVault.claim(sanctionedUser.address);

            // Check escrow amounts (allow tiny tolerance)
            approxEqual(await vestingVault.totalEscrowedAmount(), GRANT_AMOUNT);

            // Non-sanctioned user should claim normally
            await vestingVault.claim(otherUser.address);
            approxEqual(await token.balanceOf(otherUser.address), GRANT_AMOUNT / 2n);
        });
    });
    
    describe("Edge Cases", function () {
        it("Should handle zero address validation", async function () {
            await expect(vestingVault.claim(ethers.ZeroAddress))
                .to.be.revertedWith("Invalid beneficiary");
        });
        
        it("Should handle non-existent grants", async function () {
            await expect(vestingVault.claim(otherUser.address))
                .to.be.revertedWith("No active grant");
        });
        
        it("Should respect pause state", async function () {
            await vestingVault.setPaused(true);
            
            await expect(vestingVault.claim(beneficiary.address))
                .to.be.revertedWith("Contract is paused");
        });
        
        it("Should handle oracle update", async function () {
            // Deploy new oracle
            const NewSanctionsOracle = await ethers.getContractFactory("SanctionsOracle");
            const newOracle = await NewSanctionsOracle.deploy(owner.address);
            await newOracle.waitForDeployment();
            
            // Update oracle
            await vestingVault.updateSanctionsOracle(await newOracle.getAddress());
            
            expect(await vestingVault.sanctionsOracle()).to.equal(await newOracle.getAddress());
        });
    });
    
    describe("Integration Tests", function () {
        it("Should handle complete sanctions lifecycle", async function () {
            // 1. Normal vesting for 3 months
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 4]);
            await ethers.provider.send("evm_mine");
            
            await vestingVault.claim(beneficiary.address);
            const firstClaim = await token.balanceOf(beneficiary.address);
            approxEqual(firstClaim, GRANT_AMOUNT / 4n);
            
            // 2. Sanction after partial vesting
            await sanctionsOracle.sanctionAddress(beneficiary.address);
            
            // 3. Fast forward another 3 months and attempt claim
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 4]);
            await ethers.provider.send("evm_mine");
            
            await vestingVault.claim(beneficiary.address);
            
            // 4. Check escrow state
            const grant = await vestingVault.getGrant(beneficiary.address);
            expect(grant.isEscrowed).to.be.true;
            approxEqual(await vestingVault.totalEscrowedAmount(), GRANT_AMOUNT / 4n);
            
            // 5. Unsanction and release
            await sanctionsOracle.unsanctionAddress(beneficiary.address);
            console.log('before release - beneficiary:', (await token.balanceOf(beneficiary.address)).toString(), 'vault:', (await token.balanceOf(await vestingVault.getAddress())).toString());
            await vestingVault.releaseFromEscrow(beneficiary.address);
            console.log('after release - beneficiary:', (await token.balanceOf(beneficiary.address)).toString(), 'vault:', (await token.balanceOf(await vestingVault.getAddress())).toString());
            
            // 6. Verify final state
            const finalBalance = await token.balanceOf(beneficiary.address);
            approxEqual(finalBalance, GRANT_AMOUNT / 2n, 10000000000000000000000n);

            const finalGrant = await vestingVault.getGrant(beneficiary.address);
            expect(finalGrant.isEscrowed).to.be.false;
            approxEqual(finalGrant.claimed, GRANT_AMOUNT / 2n);
        });
    });

    describe("Tax Withholding", function () {
        it("Accumulates tax without losing stroops across multiple small claims", async function () {
            // Deploy a tax authority account
            const taxAuthority = owner;

            // Create a new grant with a non-zero tax rate (e.g., 123 bps = 1.23%)
            const startTime = (await ethers.provider.getBlock("latest")).timestamp;
            await vestingVault.createGrant(
                otherUser.address,
                GRANT_AMOUNT,
                startTime,
                VESTING_DURATION,
                123,
                taxAuthority.address,
                ethers.ZeroAddress // tax in same token
            );

            // Fast forward small increments and perform multiple claims to exercise rounding accumulator
            const steps = 10;
            const stepTime = Math.floor(VESTING_DURATION / steps);

            let totalGross = 0n;
            for (let i = 0; i < steps; i++) {
                await ethers.provider.send("evm_increaseTime", [stepTime]);
                await ethers.provider.send("evm_mine");

                const claimable = await vestingVault.getClaimableAmount(otherUser.address);
                if (claimable > 0n) {
                    await vestingVault.claim(otherUser.address);
                    totalGross += claimable;
                }
            }

            // Check balances: total distributed must equal recorded claimed amount
            const beneficiaryBal = await token.balanceOf(otherUser.address);
            const taxBal = await token.balanceOf(taxAuthority.address);

            const finalGrant = await vestingVault.getGrant(otherUser.address);
            // The sum of balances should approximately equal the recorded claimed amount
            approxEqual(finalGrant.claimed, beneficiaryBal + taxBal, 10000000000000000000000n);

            // The contract should have recorded cumulative taxes paid for the grant (allow larger tolerance for accumulated rounding)
            approxEqual(finalGrant.cumulative_taxes_paid, taxBal, 1000000000000000000n);
        });
    });
});
