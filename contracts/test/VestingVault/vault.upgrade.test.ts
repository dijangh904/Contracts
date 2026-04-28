// test/vault.upgrade.test.ts

import { expect } from "chai";
import { ethers, upgrades } from "hardhat";

describe("Vault Upgrade V1 -> V2 (State Compatibility)", function () {
  let vaultV1: any;
  let vaultV2: any;
  let owner: any;
  let user: any;

  /**
   * Example nested mapping in V1:
   * mapping(address => mapping(uint256 => Deposit)) public deposits;
   * struct Deposit {
   *   uint256 amount;
   *   uint256 timestamp;
   * }
   */

  before(async () => {
    [owner, user] = await ethers.getSigners();

    const VaultV1 = await ethers.getContractFactory("VaultV1");

    // Deploy proxy with V1
    vaultV1 = await upgrades.deployProxy(VaultV1, [], {
      initializer: "initialize",
    });

    await vaultV1.waitForDeployment();
  });

  it("should populate V1 state with nested mappings", async () => {
    // simulate deposits
    await vaultV1.connect(user).deposit(1, {
      value: ethers.parseEther("1"),
    });

    await vaultV1.connect(user).deposit(2, {
      value: ethers.parseEther("2"),
    });

    const deposit1 = await vaultV1.deposits(user.address, 1);
    const deposit2 = await vaultV1.deposits(user.address, 2);

    expect(deposit1.amount).to.equal(ethers.parseEther("1"));
    expect(deposit2.amount).to.equal(ethers.parseEther("2"));
  });

  it("should upgrade to V2 without breaking storage", async () => {
    const VaultV2 = await ethers.getContractFactory("VaultV2");

    // Upgrade proxy
    vaultV2 = await upgrades.upgradeProxy(
      await vaultV1.getAddress(),
      VaultV2
    );

    await vaultV2.waitForDeployment();
  });

  it("should correctly read nested mappings after upgrade", async () => {
    // 🔥 Critical: Ensure no corruption or panic
    const deposit1 = await vaultV2.deposits(user.address, 1);
    const deposit2 = await vaultV2.deposits(user.address, 2);

    expect(deposit1.amount).to.equal(ethers.parseEther("1"));
    expect(deposit1.timestamp).to.be.gt(0);

    expect(deposit2.amount).to.equal(ethers.parseEther("2"));
    expect(deposit2.timestamp).to.be.gt(0);
  });

  it("should not revert when accessing deep nested state", async () => {
    // simulate multiple reads to detect serialization / panic issues
    for (let i = 1; i <= 2; i++) {
      const deposit = await vaultV2.deposits(user.address, i);

      expect(deposit.amount).to.be.gt(0);
    }
  });

  it("should allow new V2 functionality while preserving old state", async () => {
    // Example: new function in V2
    await vaultV2.connect(user).withdraw(1);

    const deposit = await vaultV2.deposits(user.address, 1);

    expect(deposit.amount).to.equal(0);
  });
});