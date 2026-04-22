const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("VestingVault with Sanctions Oracle", function () {
    let vestingVault;
    let sanctionsOracle;
    let token;
    let owner, beneficiary, sanctionedUser, otherUser;
    
    const GRANT_AMOUNT = ethers.parseEther("1000");
    const VESTING_DURATION = 365 * 24 * 60 * 60; // 1 year in seconds
    
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
            VESTING_DURATION
        );
    });
    
    describe("Normal Vesting Flow", function () {
        it("Should allow normal claiming when not sanctioned", async function () {
            // Fast forward 6 months
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 2]);
            await ethers.provider.send("evm_mine");
            
            const claimableAmount = await vestingVault.getClaimableAmount(beneficiary.address);
            expect(claimableAmount).to.equal(GRANT_AMOUNT / 2n);
            
            await expect(vestingVault.claim(beneficiary.address))
                .to.emit(vestingVault, "TokensClaimed")
                .withArgs(beneficiary.address, claimableAmount);
            
            expect(await token.balanceOf(beneficiary.address)).to.equal(claimableAmount);
        });
        
        it("Should calculate correct claimable amount over time", async function () {
            const startTime = (await ethers.provider.getBlock("latest")).timestamp;
            
            // Check at start
            expect(await vestingVault.getClaimableAmount(beneficiary.address)).to.equal(0);
            
            // Fast forward 25% of vesting period
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 4]);
            await ethers.provider.send("evm_mine");
            
            expect(await vestingVault.getClaimableAmount(beneficiary.address))
                .to.equal(GRANT_AMOUNT / 4n);
            
            // Fast forward to completion
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION * 3 / 4]);
            await ethers.provider.send("evm_mine");
            
            expect(await vestingVault.getClaimableAmount(beneficiary.address))
                .to.equal(GRANT_AMOUNT);
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
            await expect(vestingVault.claim(beneficiary.address))
                .to.emit(vestingVault, "TokensFrozen")
                .withArgs(beneficiary.address, GRANT_AMOUNT / 2n);
            
            // Check that tokens are in escrow
            const grant = await vestingVault.getGrant(beneficiary.address);
            expect(grant.isEscrowed).to.be.true;
            expect(await vestingVault.totalEscrowedAmount()).to.equal(GRANT_AMOUNT / 2n);
            
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
            await expect(vestingVault.releaseFromEscrow(beneficiary.address))
                .to.emit(vestingVault, "TokensReleased")
                .withArgs(beneficiary.address, GRANT_AMOUNT / 2n);
            
            // Check tokens were released
            expect(await token.balanceOf(beneficiary.address)).to.equal(GRANT_AMOUNT / 2n);
            
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
                VESTING_DURATION
            );
            
            await vestingVault.createGrant(
                otherUser.address,
                GRANT_AMOUNT,
                (await ethers.provider.getBlock("latest")).timestamp,
                VESTING_DURATION
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
            
            // Check escrow amounts
            expect(await vestingVault.totalEscrowedAmount()).to.equal(GRANT_AMOUNT);
            
            // Non-sanctioned user should claim normally
            await expect(vestingVault.claim(otherUser.address))
                .to.emit(vestingVault, "TokensClaimed")
                .withArgs(otherUser.address, GRANT_AMOUNT / 2n);
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
            expect(firstClaim).to.equal(GRANT_AMOUNT / 4n);
            
            // 2. Sanction after partial vesting
            await sanctionsOracle.sanctionAddress(beneficiary.address);
            
            // 3. Fast forward another 3 months and attempt claim
            await ethers.provider.send("evm_increaseTime", [VESTING_DURATION / 4]);
            await ethers.provider.send("evm_mine");
            
            await vestingVault.claim(beneficiary.address);
            
            // 4. Check escrow state
            const grant = await vestingVault.getGrant(beneficiary.address);
            expect(grant.isEscrowed).to.be.true;
            expect(await vestingVault.totalEscrowedAmount()).to.equal(GRANT_AMOUNT / 4n);
            
            // 5. Unsanction and release
            await sanctionsOracle.unsanctionAddress(beneficiary.address);
            await vestingVault.releaseFromEscrow(beneficiary.address);
            
            // 6. Verify final state
            const finalBalance = await token.balanceOf(beneficiary.address);
            expect(finalBalance).to.equal(GRANT_AMOUNT / 2n);
            
            const finalGrant = await vestingVault.getGrant(beneficiary.address);
            expect(finalGrant.isEscrowed).to.be.false;
            expect(finalGrant.claimed).to.equal(GRANT_AMOUNT / 2n);
        });
    });
});
