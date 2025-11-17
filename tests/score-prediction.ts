import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { ScorePrediction } from "../target/types/score_prediction";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { expect } from "chai";
const fs = require("fs");
const path = require("path");

describe("score-prediction", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.scorePrediction as Program<ScorePrediction>;

  // Test accounts
  const admin = provider.wallet as anchor.Wallet;
  const oracle = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const user3 = Keypair.generate();
  const unauthorizedUser = Keypair.generate();

  // Game parameters
  const seed = new BN(1345); // Use a simple constant for testing
  const initialVirtualLiquidity = new BN(10 * LAMPORTS_PER_SOL);
  const teamAName = "Team A";
  const teamBName = "Team B";

  // PDAs
  let gameStatePDA: PublicKey;
  let vaultPDA: PublicKey;
  let user1SharesPDA: PublicKey;
  let user2SharesPDA: PublicKey;
  let user3SharesPDA: PublicKey;

  before(async () => {
    // Derive PDAs
    [gameStatePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("gameState"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    [vaultPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    [user1SharesPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("matchShares"),
        seed.toArrayLike(Buffer, "le", 8),
        user1.publicKey.toBuffer(),
      ],
      program.programId
    );

    [user2SharesPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("matchShares"),
        seed.toArrayLike(Buffer, "le", 8),
        user2.publicKey.toBuffer(),
      ],
      program.programId
    );

    [user3SharesPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("matchShares"),
        seed.toArrayLike(Buffer, "le", 8),
        user3.publicKey.toBuffer(),
      ],
      program.programId
    );

    // Airdrop SOL to test accounts
    await airdrop(provider.connection, user1.publicKey, 100 * LAMPORTS_PER_SOL);
    await airdrop(provider.connection, user2.publicKey, 100 * LAMPORTS_PER_SOL);
    await airdrop(provider.connection, user3.publicKey, 100 * LAMPORTS_PER_SOL);
    await airdrop(provider.connection, oracle.publicKey, 10 * LAMPORTS_PER_SOL);
    await airdrop(
      provider.connection,
      unauthorizedUser.publicKey,
      10 * LAMPORTS_PER_SOL
    );
  });

  describe("1. Match Initialization", () => {
    it("Admin should be able to initialize a match", async () => {
      const tx = await program.methods
        .initialize(seed, initialVirtualLiquidity, teamAName, teamBName)
        .accounts({
          adminAddress: admin.publicKey,
          oracleAddress: oracle.publicKey,
          // gameState and vault are derived automatically from seed
        })
        .rpc();

      console.log("Match initialized:", tx);

      // Verify game state
      const gameState = await program.account.gameState.fetch(gameStatePDA);
      expect(gameState.seed.toString()).to.equal(seed.toString());
      expect(gameState.adminAddress.toString()).to.equal(
        admin.publicKey.toString()
      );
      expect(gameState.oracleAddress.toString()).to.equal(
        oracle.publicKey.toString()
      );
      expect(gameState.teamAName).to.equal(teamAName);
      expect(gameState.teamBName).to.equal(teamBName);
      expect(gameState.virtualTeamAPoolTokens.toString()).to.equal(
        initialVirtualLiquidity.toString()
      );
      expect(gameState.virtualTeamBPoolTokens.toString()).to.equal(
        initialVirtualLiquidity.toString()
      );
      expect(gameState.teamAScore).to.equal(0);
      expect(gameState.teamBScore).to.equal(0);
      expect(gameState.matchStatus).to.deep.equal({ notStarted: {} });
    });

    it("Should not allow re-initialization with same seed", async () => {
      try {
        await program.methods
          .initialize(seed, initialVirtualLiquidity, teamAName, teamBName)
          .accounts({
            adminAddress: admin.publicKey,
            oracleAddress: oracle.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("2. Start Match", () => {
    it("Admin should be able to start the match", async () => {
      const tx = await program.methods
        .startGame(seed)
        .accounts({
          adminAddress: admin.publicKey,
        })
        .rpc();

      console.log("Match started:", tx);

      // Verify match status
      const gameState = await program.account.gameState.fetch(gameStatePDA);
      expect(gameState.matchStatus).to.deep.equal({ live: {} });
    });

    it("Non-admin should not be able to start the match", async () => {
      const newSeed = new BN(Date.now() + 1);
      const [newGameStatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("gameState"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [newVaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      // Initialize a new match
      await program.methods
        .initialize(newSeed, initialVirtualLiquidity, teamAName, teamBName)
        .accounts({
          adminAddress: admin.publicKey,
          oracleAddress: oracle.publicKey,
        })
        .rpc();

      // Try to start with unauthorized user
      try {
        await program.methods
          .startGame(newSeed)
          .accounts({
            adminAddress: unauthorizedUser.publicKey,
          })
          .signers([unauthorizedUser])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.error.errorCode.code).to.equal("UnauthorizedAdmin");
      }
    });
  });

  describe("3. User Deposits", () => {
    it("User1 should be able to deposit SOL for Team A", async () => {
      const depositAmount = new BN(5 * LAMPORTS_PER_SOL);

      const gameStateBefore = await program.account.gameState.fetch(
        gameStatePDA
      );
      const vaultBalanceBefore = await provider.connection.getBalance(vaultPDA);

      const tx = await program.methods
        .deposit(seed, depositAmount, teamAName)
        .accounts({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      console.log("User1 deposited for Team A:", tx);

      // Verify vault balance increased
      const vaultBalanceAfter = await provider.connection.getBalance(vaultPDA);
      expect(vaultBalanceAfter - vaultBalanceBefore).to.equal(
        depositAmount.toNumber()
      );

      // Verify user shares
      const userShares = await program.account.matchShares.fetch(
        user1SharesPDA
      );
      expect(userShares.teamAShares.toNumber()).to.be.greaterThan(0);
      expect(userShares.teamBShares.toNumber()).to.equal(0);

      // Verify game state updated
      const gameStateAfter = await program.account.gameState.fetch(
        gameStatePDA
      );
      expect(gameStateAfter.totalTeamAShares.toNumber()).to.be.greaterThan(0);
      expect(gameStateAfter.vaultSolBalance.toString()).to.equal(
        depositAmount.toString()
      );
    });

    it("User2 should be able to deposit SOL for Team B", async () => {
      const depositAmount = new BN(3 * LAMPORTS_PER_SOL);

      const tx = await program.methods
        .deposit(seed, depositAmount, teamBName)
        .accounts({
          user: user2.publicKey,
        })
        .signers([user2])
        .rpc();

      console.log("User2 deposited for Team B:", tx);

      // Verify user shares
      const userShares = await program.account.matchShares.fetch(
        user2SharesPDA
      );
      expect(userShares.teamBShares.toNumber()).to.be.greaterThan(0);
      expect(userShares.teamAShares.toNumber()).to.equal(0);
    });

    it("User3 should be able to deposit for Team A (multiple deposits)", async () => {
      const depositAmount = new BN(2 * LAMPORTS_PER_SOL);

      await program.methods
        .deposit(seed, depositAmount, teamAName)
        .accounts({
          user: user3.publicKey,
        })
        .signers([user3])
        .rpc();

      console.log("User3 deposited for Team A");

      const userShares = await program.account.matchShares.fetch(
        user3SharesPDA
      );
      expect(userShares.teamAShares.toNumber()).to.be.greaterThan(0);
    });

    it("Should fail deposit with invalid team name", async () => {
      const depositAmount = new BN(1 * LAMPORTS_PER_SOL);

      try {
        await program.methods
          .deposit(seed, depositAmount, "Invalid Team")
          .accounts({
            user: user1.publicKey,
          })
          .signers([user1])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.error.errorCode.code).to.equal("InvalidTeamName");
      }
    });
  });

  describe("4. Oracle Updates Scores", () => {
    it("Oracle should be able to update scores", async () => {
      const tx = await program.methods
        .updateScore(seed, 1, 0)
        .accounts({
          oracle: oracle.publicKey,
        })
        .signers([oracle])
        .rpc();

      console.log("Score updated by oracle:", tx);

      const gameState = await program.account.gameState.fetch(gameStatePDA);
      expect(gameState.teamAScore).to.equal(1);
      expect(gameState.teamBScore).to.equal(0);
    });

    it("Non-oracle should not be able to update scores", async () => {
      try {
        await program.methods
          .updateScore(seed, 2, 0)
          .accounts({
            oracle: unauthorizedUser.publicKey,
          })
          .signers([unauthorizedUser])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.error.errorCode.code).to.equal("UnauthorizedOracle");
      }
    });

    it("Score should not be able to decrease", async () => {
      try {
        await program.methods
          .updateScore(seed, 0, 0)
          .accounts({
            oracle: oracle.publicKey,
          })
          .signers([oracle])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.error.errorCode.code).to.equal("ScoreCannotDecrease");
      }
    });

    it("Oracle should be able to update Team B score", async () => {
      await program.methods
        .updateScore(seed, 1, 1)
        .accounts({
          oracle: oracle.publicKey,
        })
        .signers([oracle])
        .rpc();

      console.log("Team B scored");

      const gameState = await program.account.gameState.fetch(gameStatePDA);
      expect(gameState.teamAScore).to.equal(1);
      expect(gameState.teamBScore).to.equal(1);
    });

    it("Oracle should update final score (Team A wins)", async () => {
      await program.methods
        .updateScore(seed, 3, 1)
        .accounts({
          oracle: oracle.publicKey,
        })
        .signers([oracle])
        .rpc();

      console.log("Final score updated: Team A wins 3-1");

      const gameState = await program.account.gameState.fetch(gameStatePDA);
      expect(gameState.teamAScore).to.equal(3);
      expect(gameState.teamBScore).to.equal(1);
    });
  });

  describe("5. End Match", () => {
    it("Should not allow ending match before it's time", async () => {
      // This test assumes admin wants control over when match ends
      // The actual logic might vary based on your requirements
    });

    it("Admin should be able to end the match", async () => {
      const vaultBalanceBefore = await provider.connection.getBalance(vaultPDA);
      const adminBalanceBefore = await provider.connection.getBalance(
        admin.publicKey
      );

      const tx = await program.methods
        .endGame(seed)
        .accounts({
          adminAddress: admin.publicKey,
        })
        .rpc();

      console.log("Match ended:", tx);

      // Verify match status
      const gameState = await program.account.gameState.fetch(gameStatePDA);
      expect(gameState.matchStatus).to.deep.equal({ ended: {} });

      // Verify 5% platform fee was transferred to admin
      const vaultBalanceAfter = await provider.connection.getBalance(vaultPDA);
      const platformFee = Math.floor((vaultBalanceBefore * 5) / 100);

      expect(vaultBalanceBefore - vaultBalanceAfter).to.be.approximately(
        platformFee,
        5000
      );
    });

    it("Non-admin should not be able to end the match", async () => {
      const newSeed = new BN(Date.now() + 2);
      const [newGameStatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("gameState"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [newVaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      // Initialize and start a new match
      await program.methods
        .initialize(newSeed, initialVirtualLiquidity, teamAName, teamBName)
        .accounts({
          adminAddress: admin.publicKey,
          oracleAddress: oracle.publicKey,
        })
        .rpc();

      await program.methods
        .startGame(newSeed)
        .accounts({
          adminAddress: admin.publicKey,
        })
        .rpc();

      // Try to end with unauthorized user
      try {
        await program.methods
          .endGame(newSeed)
          .accounts({
            adminAddress: unauthorizedUser.publicKey,
          })
          .signers([unauthorizedUser])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.error.errorCode.code).to.equal("UnauthorizedAdmin");
      }
    });
  });

  describe("6. Claim Rewards", () => {
    it("User1 (Team A bettor) should be able to claim rewards", async () => {
      const user1BalanceBefore = await provider.connection.getBalance(
        user1.publicKey
      );
      const vaultBalanceBefore = await provider.connection.getBalance(vaultPDA);

      const userSharesBefore = await program.account.matchShares.fetch(
        user1SharesPDA
      );

      const tx = await program.methods
        .claimRewards(seed)
        .accounts({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      console.log("User1 claimed rewards:", tx);

      const user1BalanceAfter = await provider.connection.getBalance(
        user1.publicKey
      );
      const reward = user1BalanceAfter - user1BalanceBefore;

      console.log(`   User1 reward: ${reward / LAMPORTS_PER_SOL} SOL`);
      expect(reward).to.be.greaterThan(0);

      // Verify shares are reset to 0
      const userSharesAfter = await program.account.matchShares.fetch(
        user1SharesPDA
      );
      expect(userSharesAfter.teamAShares.toNumber()).to.equal(0);
      expect(userSharesAfter.teamBShares.toNumber()).to.equal(0);
    });

    it("User3 (Team A bettor) should be able to claim rewards", async () => {
      const user3BalanceBefore = await provider.connection.getBalance(
        user3.publicKey
      );

      const tx = await program.methods
        .claimRewards(seed)
        .accounts({
          user: user3.publicKey,
        })
        .signers([user3])
        .rpc();

      console.log("User3 claimed rewards:", tx);

      const user3BalanceAfter = await provider.connection.getBalance(
        user3.publicKey
      );
      const reward = user3BalanceAfter - user3BalanceBefore;

      console.log(`   User3 reward: ${reward / LAMPORTS_PER_SOL} SOL`);
      expect(reward).to.be.greaterThan(0);
    });

    it("User2 (Team B bettor) should get minimal/no rewards as they lost", async () => {
      const user2BalanceBefore = await provider.connection.getBalance(
        user2.publicKey
      );

      const tx = await program.methods
        .claimRewards(seed)
        .accounts({
          user: user2.publicKey,
        })
        .signers([user2])
        .rpc();

      console.log("User2 claimed rewards (losing team):", tx);

      const user2BalanceAfter = await provider.connection.getBalance(
        user2.publicKey
      );
      const reward = user2BalanceAfter - user2BalanceBefore;

      console.log(
        `   User2 reward: ${reward / LAMPORTS_PER_SOL} SOL (lost bet)`
      );
      // User2 bet on Team B which lost, so reward should be 0 or very minimal
    });

    it("Should not allow claiming rewards before match ends", async () => {
      const newSeed = new BN(Date.now() + 3);
      const [newGameStatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("gameState"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [newVaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [newUser1SharesPDA] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("matchShares"),
          newSeed.toArrayLike(Buffer, "le", 8),
          user1.publicKey.toBuffer(),
        ],
        program.programId
      );

      // Initialize, start match and deposit
      await program.methods
        .initialize(newSeed, initialVirtualLiquidity, teamAName, teamBName)
        .accounts({
          adminAddress: admin.publicKey,
          oracleAddress: oracle.publicKey,
        })
        .rpc();

      await program.methods
        .startGame(newSeed)
        .accounts({
          adminAddress: admin.publicKey,
        })
        .rpc();

      await program.methods
        .deposit(newSeed, new BN(1 * LAMPORTS_PER_SOL), teamAName)
        .accounts({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      // Try to claim rewards before match ends
      try {
        await program.methods
          .claimRewards(newSeed)
          .accounts({
            user: user1.publicKey,
          })
          .signers([user1])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.error.errorCode.code).to.equal("MatchNotEndedYet");
      }
    });
  });

  describe("7. Edge Cases & Security", () => {
    it("Should not allow deposits when match is not live", async () => {
      const newSeed = new BN(Date.now() + 4);
      const [newGameStatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("gameState"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [newVaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [newUser1SharesPDA] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("matchShares"),
          newSeed.toArrayLike(Buffer, "le", 8),
          user1.publicKey.toBuffer(),
        ],
        program.programId
      );

      // Initialize but don't start
      await program.methods
        .initialize(newSeed, initialVirtualLiquidity, teamAName, teamBName)
        .accounts({
          adminAddress: admin.publicKey,
          oracleAddress: oracle.publicKey,
        })
        .rpc();

      try {
        await program.methods
          .deposit(newSeed, new BN(1 * LAMPORTS_PER_SOL), teamAName)
          .accounts({
            user: user1.publicKey,
          })
          .signers([user1])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.error.errorCode.code).to.equal("MatchNotLiveYet");
      }
    });

    it("Should handle score limits properly", async () => {
      const newSeed = new BN(Date.now() + 5);
      const [newGameStatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("gameState"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [newVaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), newSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initialize(newSeed, initialVirtualLiquidity, teamAName, teamBName)
        .accounts({
          adminAddress: admin.publicKey,
          oracleAddress: oracle.publicKey,
        })
        .rpc();

      await program.methods
        .startGame(newSeed)
        .accounts({
          adminAddress: admin.publicKey,
        })
        .rpc();

      try {
        await program.methods
          .updateScore(newSeed, 51, 0)
          .accounts({
            oracle: oracle.publicKey,
          })
          .signers([oracle])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.error.errorCode.code).to.equal("ScoreTooHigh");
      }
    });
  });

  describe("8. Exact Reward Calculation Test", () => {
    it("Should calculate exact rewards for two users betting at different times", async () => {
      // Setup a new game for this specific test
      const rewardTestSeed = new BN(99999);
      const testInitialLiquidity = new BN(100 * LAMPORTS_PER_SOL); // 100 SOL virtual liquidity

      const [rewardTestGameStatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("gameState"), rewardTestSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      const [rewardTestVaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), rewardTestSeed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      // Create two new users for this test
      const bettor1 = Keypair.generate();
      const bettor2 = Keypair.generate();

      await airdrop(
        provider.connection,
        bettor1.publicKey,
        100 * LAMPORTS_PER_SOL
      );
      await airdrop(
        provider.connection,
        bettor2.publicKey,
        100 * LAMPORTS_PER_SOL
      );

      const [bettor1SharesPDA] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("matchShares"),
          rewardTestSeed.toArrayLike(Buffer, "le", 8),
          bettor1.publicKey.toBuffer(),
        ],
        program.programId
      );

      const [bettor2SharesPDA] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("matchShares"),
          rewardTestSeed.toArrayLike(Buffer, "le", 8),
          bettor2.publicKey.toBuffer(),
        ],
        program.programId
      );

      // Initialize the game
      await program.methods
        .initialize(
          rewardTestSeed,
          testInitialLiquidity,
          "Barcelona",
          "Real Madrid"
        )
        .accounts({
          adminAddress: admin.publicKey,
          oracleAddress: oracle.publicKey,
        })
        .rpc();

      // Start the game
      await program.methods
        .startGame(rewardTestSeed)
        .accounts({
          adminAddress: admin.publicKey,
        })
        .rpc();

      // --- PHASE 1: Bettor1 deposits BEFORE any goals ---
      const bettor1Amount = new BN(10 * LAMPORTS_PER_SOL); // 10 SOL

      const gameStateBeforeBet1 = await program.account.gameState.fetch(
        rewardTestGameStatePDA
      );
      const virtualA_before1 =
        gameStateBeforeBet1.virtualTeamAPoolTokens.toNumber();
      const virtualB_before1 =
        gameStateBeforeBet1.virtualTeamBPoolTokens.toNumber();
      const k_before1 = gameStateBeforeBet1.k;

      await program.methods
        .deposit(rewardTestSeed, bettor1Amount, "Barcelona")
        .accounts({
          user: bettor1.publicKey,
        })
        .signers([bettor1])
        .rpc();

      const bettor1Shares = await program.account.matchShares.fetch(
        bettor1SharesPDA
      );
      const bettor1SharesAmount = bettor1Shares.teamAShares.toNumber();

      // --- Update score: Barcelona scores ---
      await program.methods
        .updateScore(rewardTestSeed, 1, 0)
        .accounts({
          oracle: oracle.publicKey,
        })
        .signers([oracle])
        .rpc();

      const gameStateAfterGoal = await program.account.gameState.fetch(
        rewardTestGameStatePDA
      );

      // --- PHASE 2: Bettor2 deposits AFTER Barcelona scored ---
      const bettor2Amount = new BN(5 * LAMPORTS_PER_SOL); // 5 SOL

      const gameStateBeforeBet2 = await program.account.gameState.fetch(
        rewardTestGameStatePDA
      );
      const virtualA_before2 =
        gameStateBeforeBet2.virtualTeamAPoolTokens.toNumber();
      const virtualB_before2 =
        gameStateBeforeBet2.virtualTeamBPoolTokens.toNumber();

      await program.methods
        .deposit(rewardTestSeed, bettor2Amount, "Barcelona")
        .accounts({
          user: bettor2.publicKey,
        })
        .signers([bettor2])
        .rpc();

      const bettor2Shares = await program.account.matchShares.fetch(
        bettor2SharesPDA
      );
      const bettor2SharesAmount = bettor2Shares.teamAShares.toNumber();

      // --- End the game (Barcelona wins 1-0) ---
      const vaultBalanceBeforeEnd = await provider.connection.getBalance(
        rewardTestVaultPDA
      );

      await program.methods
        .endGame(rewardTestSeed)
        .accounts({
          adminAddress: admin.publicKey,
        })
        .rpc();

      const vaultBalanceAfterEnd = await provider.connection.getBalance(
        rewardTestVaultPDA
      );
      const platformFee = vaultBalanceBeforeEnd - vaultBalanceAfterEnd;

      // --- Calculate expected rewards ---
      const finalGameState = await program.account.gameState.fetch(
        rewardTestGameStatePDA
      );
      const totalWinningShares = finalGameState.totalTeamAShares.toNumber();

      // Calculate expected rewards
      const expectedBettor1Reward = Math.floor(
        (bettor1SharesAmount * vaultBalanceAfterEnd) / totalWinningShares
      );
      const expectedBettor2Reward = Math.floor(
        (bettor2SharesAmount * vaultBalanceAfterEnd) / totalWinningShares
      );

      // Calculate ROI
      const bettor1ROI =
        (expectedBettor1Reward / bettor1Amount.toNumber() - 1) * 100;
      const bettor2ROI =
        (expectedBettor2Reward / bettor2Amount.toNumber() - 1) * 100;

      // --- Claim rewards and verify ---
      const bettor1BalanceBefore = await provider.connection.getBalance(
        bettor1.publicKey
      );
      await program.methods
        .claimRewards(rewardTestSeed)
        .accounts({
          user: bettor1.publicKey,
        })
        .signers([bettor1])
        .rpc();
      const bettor1BalanceAfter = await provider.connection.getBalance(
        bettor1.publicKey
      );
      const actualBettor1Reward = bettor1BalanceAfter - bettor1BalanceBefore;

      const bettor2BalanceBefore = await provider.connection.getBalance(
        bettor2.publicKey
      );
      await program.methods
        .claimRewards(rewardTestSeed)
        .accounts({
          user: bettor2.publicKey,
        })
        .signers([bettor2])
        .rpc();
      const bettor2BalanceAfter = await provider.connection.getBalance(
        bettor2.publicKey
      );
      const actualBettor2Reward = bettor2BalanceAfter - bettor2BalanceBefore;

      // Verify rewards match expectations (with tolerance for rent-exempt balance and rounding)
      // Vault must maintain minimum rent-exempt balance, so there may be a small difference
      const tolerance = 1000000; // 0.001 SOL tolerance for rent-exempt balance

      expect(
        Math.abs(actualBettor1Reward - expectedBettor1Reward)
      ).to.be.lessThan(tolerance);

      expect(
        Math.abs(actualBettor2Reward - expectedBettor2Reward)
      ).to.be.lessThan(tolerance);

      // Verify that betting early gave better returns
      expect(bettor1ROI).to.be.greaterThan(
        bettor2ROI,
        "Early bettor should have higher ROI than late bettor"
      );
    });
  });
});

// Helper function for airdrops
async function airdrop(connection: any, publicKey: PublicKey, amount: number) {
  const signature = await connection.requestAirdrop(publicKey, amount);
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature,
    ...latestBlockhash,
  });
}
