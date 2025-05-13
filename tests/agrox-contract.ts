import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AgroxContract } from "../target/types/agrox_contract";
import { expect } from "chai";

describe("agrox-contract", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.agroxContract as Program<AgroxContract>;
  const provider = anchor.getProvider();
  const wallet = provider.wallet;

  // Generate a new keypair for the system state account
  const systemStateKeypair = anchor.web3.Keypair.generate();
  // Generate a new keypair for the machine account
  const machineKeypair = anchor.web3.Keypair.generate();
  // Generate a new keypair for the data account
  const dataKeypair = anchor.web3.Keypair.generate();
  // Generate a new keypair for the image account
  const imageKeypair = anchor.web3.Keypair.generate();

  const machineId = "IOT-001";

  it("Initializes the system", async () => {
    const tx = await program.methods
      .initialize()
      .accounts({
        systemState: systemStateKeypair.publicKey,
        authority: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([systemStateKeypair])
      .rpc();
    
    console.log("System initialized tx:", tx);
    
    // Fetch the system state account and verify initialization
    const systemState = await program.account.systemState.fetch(systemStateKeypair.publicKey);
    expect(systemState.authority.toString()).to.equal(wallet.publicKey.toString());
    expect(systemState.machineCount.toNumber()).to.equal(0);
  });

  it("Registers a new machine", async () => {
    const tx = await program.methods
      .registerMachine(machineId)
      .accounts({
        systemState: systemStateKeypair.publicKey,
        machine: machineKeypair.publicKey,
        user: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([machineKeypair])
      .rpc();
    
    console.log("Machine registered tx:", tx);
    
    // Fetch the machine account and verify initialization
    const machine = await program.account.machine.fetch(machineKeypair.publicKey);
    expect(machine.owner.toString()).to.equal(wallet.publicKey.toString());
    expect(machine.machineId).to.equal(machineId);
    expect(machine.isActive).to.equal(false);
    
    // Fetch the system state account and verify updated machine count
    const systemState = await program.account.systemState.fetch(systemStateKeypair.publicKey);
    expect(systemState.machineCount.toNumber()).to.equal(1);
  });

  it("Starts the machine", async () => {
    const tx = await program.methods
      .startMachine()
      .accounts({
        machine: machineKeypair.publicKey,
        user: wallet.publicKey,
      })
      .rpc();
    
    console.log("Machine started tx:", tx);
    
    // Fetch the machine account and verify activation
    const machine = await program.account.machine.fetch(machineKeypair.publicKey);
    expect(machine.isActive).to.equal(true);
  });

  it("Uploads sensor data with no image", async () => {
    const temperature = 25.5;
    const humidity = 60.0;
    
    const tx = await program.methods
      .uploadData(temperature, humidity, null)
      .accounts({
        systemState: systemStateKeypair.publicKey,
        machine: machineKeypair.publicKey,
        data: dataKeypair.publicKey,
        uploader: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([dataKeypair])
      .rpc();
    
    console.log("Data uploaded tx:", tx);
    
    // Fetch the data account and verify values
    const data = await program.account.ioTData.fetch(dataKeypair.publicKey);
    expect(data.machine.toString()).to.equal(machineKeypair.publicKey.toString());
    expect(data.temperature).to.equal(temperature);
    expect(data.humidity).to.equal(humidity);
    expect(data.imageUrl).to.equal(null);
    
    // Verify machine and system state updates
    const machine = await program.account.machine.fetch(machineKeypair.publicKey);
    expect(machine.dataCount.toNumber()).to.equal(1);
    expect(machine.imageCount.toNumber()).to.equal(0);
    
    const systemState = await program.account.systemState.fetch(systemStateKeypair.publicKey);
    expect(systemState.totalDataUploads.toNumber()).to.equal(1);
  });

  it("Uploads sensor data with image", async () => {
    // Create a new keypair for this data entry
    const dataWithImageKeypair = anchor.web3.Keypair.generate();
    
    const temperature = 26.5;
    const humidity = 62.0;
    const imageUrl = "https://example.com/image.jpg";
    
    const tx = await program.methods
      .uploadData(temperature, humidity, imageUrl)
      .accounts({
        systemState: systemStateKeypair.publicKey,
        machine: machineKeypair.publicKey,
        data: dataWithImageKeypair.publicKey,
        uploader: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([dataWithImageKeypair])
      .rpc();
    
    console.log("Data with image uploaded tx:", tx);
    
    // Fetch the data account and verify values
    const data = await program.account.ioTData.fetch(dataWithImageKeypair.publicKey);
    expect(data.machine.toString()).to.equal(machineKeypair.publicKey.toString());
    expect(data.temperature).to.equal(temperature);
    expect(data.humidity).to.equal(humidity);
    expect(data.imageUrl).to.equal(imageUrl);
    
    // Verify machine updates - should increment both data and image counts
    const machine = await program.account.machine.fetch(machineKeypair.publicKey);
    expect(machine.dataCount.toNumber()).to.equal(2);
    expect(machine.imageCount.toNumber()).to.equal(1);
  });

  it("Uses data and increases rewards", async () => {
    const initialMachine = await program.account.machine.fetch(machineKeypair.publicKey);
    const initialRewards = initialMachine.rewardsEarned.toNumber();
    
    const tx = await program.methods
      .useData()
      .accounts({
        systemState: systemStateKeypair.publicKey,
        machine: machineKeypair.publicKey,
        data: dataKeypair.publicKey,
        user: wallet.publicKey,
      })
      .rpc();
    
    console.log("Data used tx:", tx);
    
    // Verify data usage count and rewards increase
    const data = await program.account.ioTData.fetch(dataKeypair.publicKey);
    expect(data.usedCount.toNumber()).to.equal(1);
    
    const machine = await program.account.machine.fetch(machineKeypair.publicKey);
    expect(machine.dataUsedCount.toNumber()).to.equal(1);
    expect(machine.rewardsEarned.toNumber()).to.be.greaterThan(initialRewards);
  });

  it("Claims rewards", async () => {
    const tx = await program.methods
      .claimRewards()
      .accounts({
        machine: machineKeypair.publicKey,
        user: wallet.publicKey,
      })
      .rpc();
    
    console.log("Rewards claimed tx:", tx);
    
    // Verify rewards are reset
    const machine = await program.account.machine.fetch(machineKeypair.publicKey);
    expect(machine.rewardsEarned.toNumber()).to.equal(0);
  });

  it("Stops the machine", async () => {
    const tx = await program.methods
      .stopMachine()
      .accounts({
        machine: machineKeypair.publicKey,
        user: wallet.publicKey,
      })
      .rpc();
    
    console.log("Machine stopped tx:", tx);
    
    // Verify machine is stopped
    const machine = await program.account.machine.fetch(machineKeypair.publicKey);
    expect(machine.isActive).to.equal(false);
  });
});
