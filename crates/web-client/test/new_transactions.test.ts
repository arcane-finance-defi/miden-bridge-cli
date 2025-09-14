import test from "./playwright.global.setup";
import { expect, Page } from "@playwright/test";
import {
  consumeTransaction,
  mintAndConsumeTransaction,
  mintTransaction,
  sendTransaction,
  setupWalletAndFaucet,
  swapTransaction,
  setupConsumedNote,
} from "./webClientTestUtils";
import {
  Account,
  TransactionRecord,
  Note,
} from "../dist/crates/miden_client_web";

// NEW_MINT_TRANSACTION TESTS
// =======================================================================================================

interface MultipleMintsTransactionResult {
  transactionIds: string[];
  createdNoteIds: string[];
  numOutputNotesCreated: number;
  nonce: string | undefined;
  finalBalance: string | undefined;
}

const multipleMintsTest = async (
  testingPage: Page,
  targetAccount: string,
  faucetAccount: string,
  withRemoteProver: boolean = false
): Promise<MultipleMintsTransactionResult> => {
  return await testingPage.evaluate(
    async ({ targetAccount, faucetAccount, withRemoteProver }) => {
      const client = window.client;

      const targetAccountId = window.AccountId.fromHex(targetAccount);
      const faucetAccountId = window.AccountId.fromHex(faucetAccount);
      await client.syncState();

      // Mint 3 notes
      let result: {
        transactionIds: string[];
        createdNoteIds: string[];
        numOutputNotesCreated: number;
      } = {
        transactionIds: [],
        createdNoteIds: [],
        numOutputNotesCreated: 0,
      };

      for (let i = 0; i < 3; i++) {
        const mintTransactionRequest = await client.newMintTransactionRequest(
          targetAccountId,
          faucetAccountId,
          window.NoteType.Public,
          BigInt(1000)
        );

        const mintTransactionResult = await client.newTransaction(
          faucetAccountId,
          mintTransactionRequest
        );

        if (withRemoteProver && window.remoteProverUrl != null) {
          await client.submitTransaction(
            mintTransactionResult,
            window.remoteProverInstance
          );
        } else {
          await client.submitTransaction(mintTransactionResult);
        }

        await window.helpers.waitForTransaction(
          mintTransactionResult.executedTransaction().id().toHex()
        );

        result.createdNoteIds.push(
          mintTransactionResult.createdNotes().notes()[0].id().toString()
        );
        result.transactionIds.push(
          mintTransactionResult.executedTransaction().id().toHex()
        );
        result.numOutputNotesCreated += mintTransactionResult
          .createdNotes()
          .numNotes();
      }

      // Consume the minted notes
      for (let i = 0; i < result.createdNoteIds.length; i++) {
        const consumeTransactionRequest = client.newConsumeTransactionRequest([
          result.createdNoteIds[i],
        ]);
        const consumeTransactionResult = await client.newTransaction(
          targetAccountId,
          consumeTransactionRequest
        );

        if (withRemoteProver && window.remoteProverUrl != null) {
          await client.submitTransaction(
            consumeTransactionResult,
            window.remoteProverInstance
          );
        } else {
          await client.submitTransaction(consumeTransactionResult);
        }

        await window.helpers.waitForTransaction(
          consumeTransactionResult.executedTransaction().id().toHex()
        );
      }

      const changedTargetAccount = await client.getAccount(targetAccountId);

      return {
        ...result,
        nonce: changedTargetAccount!.nonce()?.toString(),
        finalBalance: changedTargetAccount!
          .vault()
          .getBalance(faucetAccountId)
          .toString(),
      };
    },
    {
      targetAccount,
      faucetAccount,
      withRemoteProver,
    }
  );
};

test.describe("mint transaction tests", () => {
  const testCases = [
    { flag: false, description: "mint transaction completes successfully" },
    {
      flag: true,
      description: "mint transaction with remote prover completes successfully",
    },
  ];

  testCases.forEach(({ flag, description }) => {
    test(description, async ({ page }) => {
      // This test was added in #995 to reproduce an issue in the web wallet.
      // It is useful because most tests consume the note right on the latest client block,
      // but this test mints 3 notes and consumes them after the fact. This ensures the
      // MMR data for old blocks is available and valid so that the notes can be consumed.
      const { faucetId, accountId } = await setupWalletAndFaucet(page);
      const result = await multipleMintsTest(page, accountId, faucetId, flag);

      expect(result.transactionIds.length).toEqual(3);
      expect(result.numOutputNotesCreated).toEqual(3);
      expect(result.nonce).toEqual("3");
      expect(result.finalBalance).toEqual("3000");
      expect(result.createdNoteIds.length).toEqual(3);
    });
  });
});

// NEW_CONSUME_TRANSACTION TESTS
// =======================================================================================================

test.describe("consume transaction tests", () => {
  const testCases = [
    { flag: false, description: "consume transaction completes successfully" },
    {
      flag: true,
      description:
        "consume transaction with remote prover completes successfully",
    },
  ];

  testCases.forEach(({ flag, description }) => {
    test(description, async ({ page }) => {
      const { faucetId, accountId } = await setupWalletAndFaucet(page);
      const { consumeResult: result } = await mintAndConsumeTransaction(
        page,
        accountId,
        faucetId,
        flag
      );

      expect(result.transactionId).toHaveLength(66);
      expect(result.nonce).toEqual("1");
      expect(result.numConsumedNotes).toEqual(1);
      expect(result.targetAccountBalance).toEqual("1000");
    });
  });
});

// NEW_SEND_TRANSACTION TESTS
// =======================================================================================================

interface SendTransactionResult {
  senderAccountBalance: string;
  changedTargetBalance: string;
}

export const sendTransactionTest = async (
  testingPage: Page,
  senderAccount: string,
  targetAccount: string,
  faucetAccount: string
): Promise<SendTransactionResult> => {
  return await testingPage.evaluate(
    async ({ senderAccount, targetAccount, faucetAccount }) => {
      const client = window.client;

      await client.syncState();

      const targetAccountId = window.AccountId.fromHex(targetAccount);
      const senderAccountId = window.AccountId.fromHex(senderAccount);
      const faucetAccountId = window.AccountId.fromHex(faucetAccount);

      const changedSenderAccount = await client.getAccount(senderAccountId);
      const changedTargetAccount = await client.getAccount(targetAccountId);

      return {
        senderAccountBalance: changedSenderAccount!
          .vault()
          .getBalance(faucetAccountId)
          .toString(),
        changedTargetBalance: changedTargetAccount!
          .vault()
          .getBalance(faucetAccountId)
          .toString(),
      };
    },
    {
      senderAccount,
      targetAccount,
      faucetAccount,
    }
  );
};

test.describe("send transaction tests", () => {
  const testCases = [
    { flag: false, description: "send transaction completes successfully" },
    {
      flag: true,
      description: "send transaction with remote prover completes successfully",
    },
  ];

  testCases.forEach(({ flag, description }) => {
    test(description, async ({ page }) => {
      const { accountId: senderAccountId, faucetId } =
        await setupWalletAndFaucet(page);
      const { accountId: targetAccountId } = await setupWalletAndFaucet(page);
      const recallHeight = 100;
      let createdSendNotes = await sendTransaction(
        page,
        senderAccountId,
        targetAccountId,
        faucetId,
        recallHeight,
        flag
      );

      await consumeTransaction(
        page,
        targetAccountId,
        faucetId,
        createdSendNotes[0]
      );
      const result = await sendTransactionTest(
        page,
        senderAccountId,
        targetAccountId,
        faucetId
      );

      expect(result.senderAccountBalance).toEqual("900");
      expect(result.changedTargetBalance).toEqual("100");
    });
  });
});

// SWAP_TRANSACTION TEST
// =======================================================================================================

test.describe("swap transaction tests", () => {
  const testCases = [
    { flag: false, description: "swap transaction completes successfully" },
    {
      flag: true,
      description: "swap transaction with remote prover completes successfully",
    },
  ];

  testCases.forEach(({ flag, description }) => {
    test(description, async ({ page }) => {
      const { accountId: accountA, faucetId: faucetA } =
        await setupWalletAndFaucet(page);
      const { accountId: accountB, faucetId: faucetB } =
        await setupWalletAndFaucet(page);

      const assetAAmount = BigInt(1);
      const assetBAmount = BigInt(25);

      await mintAndConsumeTransaction(page, accountA, faucetA, flag);
      await mintAndConsumeTransaction(page, accountB, faucetB, flag);

      const { accountAAssets, accountBAssets } = await swapTransaction(
        page,
        accountA,
        accountB,
        faucetA,
        assetAAmount,
        faucetB,
        assetBAmount,
        "private",
        "private",
        flag
      );

      // --- assertions for Account A ---
      const aA = accountAAssets!.find((a) => a.assetId === faucetA);
      expect(aA, `Expected to find asset ${faucetA} on Account A`).toBeTruthy();
      expect(BigInt(aA!.amount)).toEqual(999n);

      const aB = accountAAssets!.find((a) => a.assetId === faucetB);
      expect(aB, `Expected to find asset ${faucetB} on Account A`).toBeTruthy();
      expect(BigInt(aB!.amount)).toEqual(25n);

      // --- assertions for Account B ---
      const bA = accountBAssets!.find((a) => a.assetId === faucetA);
      expect(bA, `Expected to find asset ${faucetA} on Account B`).toBeTruthy();
      expect(BigInt(bA!.amount)).toEqual(1n);

      const bB = accountBAssets!.find((a) => a.assetId === faucetB);
      expect(bB, `Expected to find asset ${faucetB} on Account B`).toBeTruthy();
      expect(BigInt(bB!.amount)).toEqual(975n);
    });
  });
});

// CUSTOM_TRANSACTIONS TESTS
// =======================================================================================================

export const customTransaction = async (
  testingPage: Page,
  assertedValue: string,
  withRemoteProver: boolean
): Promise<boolean> => {
  return await testingPage.evaluate(
    async ({ assertedValue, withRemoteProver }) => {
      debugger;
      const client = window.client;

      const walletAccount = await client.newWallet(
        window.AccountStorageMode.private(),
        false
      );
      const faucetAccount = await client.newFaucet(
        window.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        BigInt(10000000)
      );
      await client.syncState();

      // Creating Custom Note which needs the following:
      // - Note Assets
      // - Note Metadata
      // - Note Recipient

      // Creating NOTE_ARGS
      let felt1 = new window.Felt(BigInt(9));
      let felt2 = new window.Felt(BigInt(12));
      let felt3 = new window.Felt(BigInt(18));
      let felt4 = new window.Felt(BigInt(3));
      let felt5 = new window.Felt(BigInt(3));
      let felt6 = new window.Felt(BigInt(18));
      let felt7 = new window.Felt(BigInt(12));
      let felt8 = new window.Felt(BigInt(9));

      let noteArgs = [felt1, felt2, felt3, felt4, felt5, felt6, felt7, felt8];
      let feltArray = new window.FeltArray();
      noteArgs.forEach((felt) => feltArray.append(felt));

      let noteAssets = new window.NoteAssets([
        new window.FungibleAsset(faucetAccount.id(), BigInt(10)),
      ]);

      let noteMetadata = new window.NoteMetadata(
        faucetAccount.id(),
        window.NoteType.Private,
        window.NoteTag.fromAccountId(walletAccount.id()),
        window.NoteExecutionHint.none(),
        undefined
      );

      let expectedNoteArgs = noteArgs.map((felt) => felt.asInt());
      let memAddress = "1000";
      let memAddress2 = "1004";
      let expectedNoteArg1 = expectedNoteArgs.slice(0, 4).join(".");
      let expectedNoteArg2 = expectedNoteArgs.slice(4, 8).join(".");
      let noteScript = `
            # Custom P2ID note script
            #
            # This note script asserts that the note args are exactly the same as passed
            # (currently defined as {expected_note_arg_1} and {expected_note_arg_2}).
            # Since the args are too big to fit in a single note arg, we provide them via advice inputs and
            # address them via their commitment (noted as NOTE_ARG)
            # This note script is based off of the P2ID note script because notes currently need to have
            # assets, otherwise it could have been boiled down to the assert.

            use.miden::account
            use.miden::note
            use.miden::contracts::wallets::basic->wallet
            use.std::mem


            proc.add_note_assets_to_account
                push.0 exec.note::get_assets
                # => [num_of_assets, 0 = ptr, ...]

                # compute the pointer at which we should stop iterating
                mul.4 dup.1 add
                # => [end_ptr, ptr, ...]

                # pad the stack and move the pointer to the top
                padw movup.5
                # => [ptr, 0, 0, 0, 0, end_ptr, ...]

                # compute the loop latch
                dup dup.6 neq
                # => [latch, ptr, 0, 0, 0, 0, end_ptr, ...]

                while.true
                    # => [ptr, 0, 0, 0, 0, end_ptr, ...]

                    # save the pointer so that we can use it later
                    dup movdn.5
                    # => [ptr, 0, 0, 0, 0, ptr, end_ptr, ...]

                    # load the asset
                    mem_loadw
                    # => [ASSET, ptr, end_ptr, ...]

                    # pad the stack before call
                    padw swapw padw padw swapdw
                    # => [ASSET, pad(12), ptr, end_ptr, ...]

                    # add asset to the account
                    call.wallet::receive_asset
                    # => [pad(16), ptr, end_ptr, ...]

                    # clean the stack after call
                    dropw dropw dropw
                    # => [0, 0, 0, 0, ptr, end_ptr, ...]

                    # increment the pointer and compare it to the end_ptr
                    movup.4 add.4 dup dup.6 neq
                    # => [latch, ptr+4, ASSET, end_ptr, ...]
                end

                # clear the stack
                drop dropw drop
            end

            begin
                # push data from the advice map into the advice stack
                adv.push_mapval
                # => [NOTE_ARG]

                # memory address where to write the data
                push.${memAddress}
                # => [target_mem_addr, NOTE_ARG_COMMITMENT]
                # number of words
                push.2
                # => [number_of_words, target_mem_addr, NOTE_ARG_COMMITMENT]
                exec.mem::pipe_preimage_to_memory
                # => [target_mem_addr']
                dropw
                # => []

                # read first word
                push.${memAddress}
                # => [data_mem_address]
                mem_loadw
                # => [NOTE_ARG_1]

                push.${expectedNoteArg1} assert_eqw.err="First note argument didn't match expected"
                # => []

                # read second word
                push.${memAddress2}
                # => [data_mem_address_2]
                mem_loadw
                # => [NOTE_ARG_2]

                push.${expectedNoteArg2} assert_eqw.err="Second note argument didn't match expected"
                # => []

                # store the note inputs to memory starting at address 0
                push.0 exec.note::get_inputs
                # => [num_inputs, inputs_ptr]

                # make sure the number of inputs is 2
                eq.2 assert.err="P2ID script expects exactly 2 note inputs"
                # => [inputs_ptr]

                # read the target account id from the note inputs
                mem_load
                # => [target_account_id_prefix]

                exec.account::get_id swap drop
                # => [account_id_prefix, target_account_id_prefix, ...]

                # ensure account_id = target_account_id, fails otherwise
                assert_eq.err="P2ID's target account address and transaction address do not match"
                # => [...]

                exec.add_note_assets_to_account
                # => [...]
            end
        `;

      let compiledNoteScript = await client.compileNoteScript(noteScript);
      let noteInputs = new window.NoteInputs(
        new window.FeltArray([
          walletAccount.id().prefix(),
          walletAccount.id().suffix(),
        ])
      );

      const serialNum = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      let noteRecipient = new window.NoteRecipient(
        serialNum,
        compiledNoteScript,
        noteInputs
      );

      let note = new window.Note(noteAssets, noteMetadata, noteRecipient);

      // Creating First Custom Transaction Request to Mint the Custom Note
      let transactionRequest = new window.TransactionRequestBuilder()
        .withOwnOutputNotes(
          new window.OutputNotesArray([window.OutputNote.full(note)])
        )
        .build();

      // Execute and Submit Transaction
      let transactionResult = await client.newTransaction(
        faucetAccount.id(),
        transactionRequest
      );

      if (withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          transactionResult,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(transactionResult);
      }

      await window.helpers.waitForTransaction(
        transactionResult.executedTransaction().id().toHex()
      );

      // Just like in the miden test, you can modify this script to get the execution to fail
      // by modifying the assert
      let txScript = `
            use.miden::kernels::tx::prologue
            use.miden::kernels::tx::memory

            begin
                push.0 push.${assertedValue}
                # => [0, ${assertedValue}]
                assert_eq
            end
        `;

      // Creating Second Custom Transaction Request to Consume Custom Note
      // with Invalid/Valid Transaction Script
      let transactionScript = await client.compileTxScript(txScript);
      let noteArgsCommitment = window.Rpo256.hashElements(feltArray); // gets consumed by NoteIdAndArgs
      let noteAndArgs = new window.NoteAndArgs(note, noteArgsCommitment);
      let noteAndArgsArray = new window.NoteAndArgsArray([noteAndArgs]);
      let adviceMap = new window.AdviceMap();
      let noteArgsCommitment2 = window.Rpo256.hashElements(feltArray);
      adviceMap.insert(noteArgsCommitment2, feltArray);

      let transactionRequest2 = new window.TransactionRequestBuilder()
        .withUnauthenticatedInputNotes(noteAndArgsArray)
        .withCustomScript(transactionScript)
        .extendAdviceMap(adviceMap)
        .build();

      // Execute and Submit Transaction
      let transactionResult2 = await client.newTransaction(
        walletAccount.id(),
        transactionRequest2
      );

      if (withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          transactionResult2,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(transactionResult2);
      }

      await window.helpers.waitForTransaction(
        transactionResult2.executedTransaction().id().toHex()
      );
    },
    {
      assertedValue,
      withRemoteProver,
    }
  );
};

const customTxWithMultipleNotes = async (
  testingPage: Page,
  isSerialNumSame: boolean,
  senderAccountId: string,
  faucetAccountId: string
) => {
  return await testingPage.evaluate(
    async ({ isSerialNumSame, _senderAccountId, _faucetAccountId }) => {
      const client = window.client;
      const amount = BigInt(10);
      const targetAccount = await client.newWallet(
        window.AccountStorageMode.private(),
        true
      );
      const targetAccountId = targetAccount.id();
      const senderAccountId = window.AccountId.fromHex(_senderAccountId);
      const faucetAccountId = window.AccountId.fromHex(_faucetAccountId);

      // Create custom note with multiple assets to send to target account
      // Error should happen if serial numbers are the same in each set of
      // note assets. Otherwise, the transaction should go through.

      let noteAssets1 = new window.NoteAssets([
        new window.FungibleAsset(faucetAccountId, amount),
      ]);
      let noteAssets2 = new window.NoteAssets([
        new window.FungibleAsset(faucetAccountId, amount),
      ]);

      let noteMetadata = new window.NoteMetadata(
        senderAccountId,
        window.NoteType.Public,
        window.NoteTag.fromAccountId(targetAccountId),
        window.NoteExecutionHint.none(),
        undefined
      );

      let serialNum1 = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      let serialNum2 = new window.Word(
        new BigUint64Array([BigInt(5), BigInt(6), BigInt(7), BigInt(8)])
      );

      const p2idScript = window.NoteScript.p2id();

      let noteInputs = new window.NoteInputs(
        new window.FeltArray([
          targetAccount.id().suffix(),
          targetAccount.id().prefix(),
        ])
      );

      let noteRecipient1 = new window.NoteRecipient(
        serialNum1,
        p2idScript,
        noteInputs
      );
      let noteRecipient2 = new window.NoteRecipient(
        isSerialNumSame ? serialNum1 : serialNum2,
        p2idScript,
        noteInputs
      );

      let note1 = new window.Note(noteAssets1, noteMetadata, noteRecipient1);
      let note2 = new window.Note(noteAssets2, noteMetadata, noteRecipient2);

      let transactionRequest = new window.TransactionRequestBuilder()
        .withOwnOutputNotes(
          new window.OutputNotesArray([
            window.OutputNote.full(note1),
            window.OutputNote.full(note2),
          ])
        )
        .build();

      let transactionResult = await client.newTransaction(
        senderAccountId,
        transactionRequest
      );

      await client.submitTransaction(transactionResult);

      await window.helpers.waitForTransaction(
        transactionResult.executedTransaction().id().toHex()
      );
    },
    {
      isSerialNumSame,
      _senderAccountId: senderAccountId,
      _faucetAccountId: faucetAccountId,
    }
  );
};

test.describe("custom transaction tests", () => {
  test("custom transaction completes successfully", async ({ page }) => {
    await expect(customTransaction(page, "0", false)).resolves.toBeUndefined();
  });

  test("custom transaction fails", async ({ page }) => {
    await expect(customTransaction(page, "1", false)).rejects.toThrow();
  });

  test("custom transaction with remote prover completes successfully", async ({
    page,
  }) => {
    await expect(customTransaction(page, "0", true)).resolves.toBeUndefined();
  });
});

test.describe("custom transaction with multiple output notes", () => {
  const testCases = [
    {
      description: "does not fail when output note serial numbers are unique",
      shouldFail: false,
    },
    {
      description: "fails when output note serial numbers are the same",
      shouldFail: true,
    },
  ];

  testCases.forEach(({ description, shouldFail }) => {
    test(description, async ({ page }) => {
      const { accountId, faucetId } = await setupConsumedNote(page);
      if (shouldFail) {
        await expect(
          customTxWithMultipleNotes(page, shouldFail, accountId, faucetId)
        ).rejects.toThrow();
      } else {
        await expect(
          customTxWithMultipleNotes(page, shouldFail, accountId, faucetId)
        ).resolves.toBeUndefined();
      }
    });
  });
});

// CUSTOM ACCOUNT COMPONENT TESTS
// =======================================================================================================

export const customAccountComponent = async (
  testingPage: Page
): Promise<void> => {
  return await testingPage.evaluate(async () => {
    const accountCode = `
        use.miden::account
        use.std::sys

        # Inputs: [KEY, VALUE]
        # Outputs: []
        export.write_to_map
            # The storage map is in storage slot 1
            push.1
            # => [index, KEY, VALUE]

            # Setting the key value pair in the map
            exec.account::set_map_item
            # => [OLD_MAP_ROOT, OLD_MAP_VALUE]

            dropw dropw dropw dropw
            # => []
        end

        # Inputs: [KEY]
        # Outputs: [VALUE]
        export.get_value_in_map
            # The storage map is in storage slot 1
            push.1
            # => [index, KEY]

            exec.account::get_map_item
            # => [VALUE]
        end

        # Inputs: []
        # Outputs: [CURRENT_ROOT]
        export.get_current_map_root
            # Getting the current root from slot 1
            push.1 exec.account::get_item
            # => [CURRENT_ROOT]

            exec.sys::truncate_stack
            # => [CURRENT_ROOT]
        end
      `;
    const scriptCode = `
        use.miden_by_example::mapping_example_contract
        use.std::sys

        begin
            push.1.2.3.4
            push.0.0.0.0
            # => [KEY, VALUE]

            call.mapping_example_contract::write_to_map
            # => []

            push.0.0.0.0
            # => [KEY]

            call.mapping_example_contract::get_value_in_map
            # => [VALUE]

            dropw
            # => []

            call.mapping_example_contract::get_current_map_root
            # => [CURRENT_ROOT]

            exec.sys::truncate_stack
        end
      `;
    const client = window.client;
    let assembler = window.TransactionKernel.assembler().withDebugMode(true);
    let emptyStorageSlot = window.StorageSlot.emptyValue();
    let storageMap = new window.StorageMap();
    let storageSlotMap = window.StorageSlot.map(storageMap);

    let mappingAccountComponent = window.AccountComponent.compile(
      accountCode,
      assembler,
      [emptyStorageSlot, storageSlotMap]
    ).withSupportsAllTypes();

    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    let secretKey = window.SecretKey.withRng(walletSeed);
    let authComponent = window.AccountComponent.createAuthComponent(secretKey);

    let accountBuilderResult = new window.AccountBuilder(walletSeed)
      .accountType(window.AccountType.RegularAccountImmutableCode)
      .storageMode(window.AccountStorageMode.public())
      .withAuthComponent(authComponent)
      .withComponent(mappingAccountComponent)
      .build();

    await client.addAccountSecretKeyToWebStore(secretKey);
    await client.newAccount(
      accountBuilderResult.account,
      accountBuilderResult.seed,
      false
    );

    await client.syncState();

    let accountComponentLib =
      window.AssemblerUtils.createAccountComponentLibrary(
        assembler,
        "miden_by_example::mapping_example_contract",
        accountCode
      );

    let txScript = window.TransactionScript.compile(
      scriptCode,
      assembler.withLibrary(accountComponentLib)
    );

    let txIncrementRequest = new window.TransactionRequestBuilder()
      .withCustomScript(txScript)
      .build();

    let txResult = await client.newTransaction(
      accountBuilderResult.account.id(),
      txIncrementRequest
    );
    await client.submitTransaction(txResult);

    await window.helpers.waitForTransaction(
      txResult.executedTransaction().id().toHex()
    );

    // Fetch the updated account state from the client
    const updated = await client.getAccount(accountBuilderResult.account.id());

    // Read a map value from storage slot 1 with key 0x0
    const keyZero = new window.Word(new BigUint64Array([0n, 0n, 0n, 0n]));
    // NOTE: the map slot is in index 2 because the auth component takes one slot
    const retrieveMapKey = updated?.storage().getMapItem(2, keyZero);

    const expected = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));

    if (retrieveMapKey?.toHex() !== expected.toHex()) {
      throw new Error(
        `unexpected Word: got ${retrieveMapKey?.toHex()} expected ${expected.toHex()}`
      );
    }
  });
};

test.describe("custom account component tests", () => {
  test("custom account component transaction completes successfully", async ({
    page,
  }) => {
    await expect(customAccountComponent(page)).resolves.toBeUndefined();
  });
});

// DISCARDED TRANSACTIONS TESTS
// ================================================================================================

interface DiscardedTransactionResult {
  discardedTransactions: TransactionRecord[];
  commitmentBeforeTx: string;
  commitmentAfterTx: string;
  commitmentAfterDiscardedTx: string;
}

export const discardedTransaction = async (
  testingPage: Page
): Promise<DiscardedTransactionResult> => {
  return await testingPage.evaluate(async () => {
    const client = window.client;

    const senderAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true
    );
    const targetAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true
    );
    const faucetAccount = await client.newFaucet(
      window.AccountStorageMode.private(),
      false,
      "DAG",
      8,
      BigInt(10000000)
    );
    await client.syncState();

    let mintTransactionRequest = client.newMintTransactionRequest(
      senderAccount.id(),
      faucetAccount.id(),
      window.NoteType.Private,
      BigInt(1000)
    );
    let mintTransactionResult = await client.newTransaction(
      faucetAccount.id(),
      mintTransactionRequest
    );
    await client.submitTransaction(mintTransactionResult);
    let createdNotes = mintTransactionResult.createdNotes().notes();
    let createdNoteIds = createdNotes.map((note: Note) => note.id().toString());
    await window.helpers.waitForTransaction(
      mintTransactionResult.executedTransaction().id().toHex()
    );

    const senderConsumeTransactionRequest =
      client.newConsumeTransactionRequest(createdNoteIds);
    let senderConsumeTransactionResult = await client.newTransaction(
      senderAccount.id(),
      senderConsumeTransactionRequest
    );
    await client.submitTransaction(senderConsumeTransactionResult);
    await window.helpers.waitForTransaction(
      senderConsumeTransactionResult.executedTransaction().id().toHex()
    );

    let sendTransactionRequest = client.newSendTransactionRequest(
      senderAccount.id(),
      targetAccount.id(),
      faucetAccount.id(),
      window.NoteType.Private,
      BigInt(100),
      1,
      null
    );
    let sendTransactionResult = await client.newTransaction(
      senderAccount.id(),
      sendTransactionRequest
    );
    await client.submitTransaction(sendTransactionResult);
    let sendCreatedNotes = sendTransactionResult.createdNotes().notes();
    let sendCreatedNoteIds = sendCreatedNotes.map((note: Note) =>
      note.id().toString()
    );

    await window.helpers.waitForTransaction(
      sendTransactionResult.executedTransaction().id().toHex()
    );

    let noteIdAndArgs = new window.NoteIdAndArgs(
      sendCreatedNotes[0].id(),
      null
    );
    let noteIdAndArgsArray = new window.NoteIdAndArgsArray([noteIdAndArgs]);
    const consumeTransactionRequest = new window.TransactionRequestBuilder()
      .withAuthenticatedInputNotes(noteIdAndArgsArray)
      .build();

    let preConsumeStore = await client.exportStore();

    // Sender retrieves the note
    let senderTxRequest =
      await client.newConsumeTransactionRequest(sendCreatedNoteIds);
    let senderTxResult = await client.newTransaction(
      senderAccount.id(),
      senderTxRequest
    );
    await client.submitTransaction(senderTxResult);
    await window.helpers.waitForTransaction(
      senderTxResult.executedTransaction().id().toHex()
    );

    await client.forceImportStore(preConsumeStore);

    // Get the account state before the transaction is applied
    const accountStateBeforeTx = (await client.getAccount(
      targetAccount.id()
    )) as Account;
    if (!accountStateBeforeTx) {
      throw new Error("Failed to get account state before transaction");
    }

    // Target tries consuming but the transaction will not be submitted
    let targetTxResult = await client.newTransaction(
      targetAccount.id(),
      consumeTransactionRequest
    );

    await client.testingApplyTransaction(targetTxResult);
    // Get the account state after the transaction is applied
    const accountStateAfterTx = (await client.getAccount(
      targetAccount.id()
    )) as Account;
    if (!accountStateAfterTx) {
      throw new Error("Failed to get account state after transaction");
    }

    await client.syncState();

    const allTransactions = await client.getTransactions(
      window.TransactionFilter.all()
    );

    const discardedTransactions = allTransactions.filter(
      (tx: TransactionRecord) => tx.transactionStatus().isDiscarded()
    );

    // Get the account state after the discarded transactions are applied
    const accountStateAfterDiscardedTx = (await client.getAccount(
      targetAccount.id()
    )) as Account;
    if (!accountStateAfterDiscardedTx) {
      throw new Error(
        "Failed to get account state after discarded transaction"
      );
    }

    // Perform a `.commitment()` check on each account
    const commitmentBeforeTx = accountStateBeforeTx.commitment().toHex();
    const commitmentAfterTx = accountStateAfterTx.commitment().toHex();
    const commitmentAfterDiscardedTx = accountStateAfterDiscardedTx
      .commitment()
      .toHex();

    return {
      discardedTransactions: discardedTransactions,
      commitmentBeforeTx,
      commitmentAfterTx,
      commitmentAfterDiscardedTx,
    };
  });
};

test.describe("discarded_transaction tests", () => {
  test("transaction gets discarded", async ({ page }) => {
    const result = await discardedTransaction(page);

    expect(result.discardedTransactions.length).toEqual(1);
    expect(result.commitmentBeforeTx).toEqual(
      result.commitmentAfterDiscardedTx
    );
    expect(result.commitmentAfterTx).not.toEqual(
      result.commitmentAfterDiscardedTx
    );
  });
});

// NETWORK TRANSACTION TESTS
// ================================================================================================

export const counterAccountComponent = async (
  testingPage: Page
): Promise<string | undefined> => {
  return await testingPage.evaluate(async () => {
    const accountCode = `
        use.miden::account
        use.std::sys

        # => []
        export.get_count
            push.0
            exec.account::get_item
            exec.sys::truncate_stack
        end

        # => []
        export.increment_count
            push.0
            # => [index]
            exec.account::get_item
            # => [count]
            push.1 add
            # => [count+1]
            push.0
            # [index, count+1]
            exec.account::set_item
            # => []
            exec.sys::truncate_stack
            # => []
        end
      `;
    const scriptCode = `
        use.external_contract::counter_contract
        begin
            call.counter_contract::increment_count
        end
      `;
    const incrNonceAuthCode = `use.miden::account
        export.auth__basic
          exec.account::incr_nonce
          drop
        end`;
    const client = window.client;

    // Create counter account
    let assembler = window.TransactionKernel.assembler().withDebugMode(true);
    let emptyStorageSlot = window.StorageSlot.emptyValue();

    let counterAccountComponent = window.AccountComponent.compile(
      accountCode,
      assembler,
      [emptyStorageSlot]
    ).withSupportsAllTypes();

    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    let incrNonceAuth = window.AccountComponent.compile(
      incrNonceAuthCode,
      assembler,
      []
    ).withSupportsAllTypes();

    let accountBuilderResult = new window.AccountBuilder(walletSeed)
      .storageMode(window.AccountStorageMode.network())
      .withAuthComponent(incrNonceAuth)
      .withComponent(counterAccountComponent)
      .build();

    await client.newAccount(
      accountBuilderResult.account,
      accountBuilderResult.seed,
      false
    );

    const nativeAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      false
    );

    await client.syncState();

    // Deploy counter account
    let accountComponentLib =
      window.AssemblerUtils.createAccountComponentLibrary(
        assembler,
        "external_contract::counter_contract",
        accountCode
      );

    let txScript = window.TransactionScript.compile(
      scriptCode,
      assembler.withLibrary(accountComponentLib)
    );

    let txIncrementRequest = new window.TransactionRequestBuilder()
      .withCustomScript(txScript)
      .build();

    let txResult = await client.newTransaction(
      accountBuilderResult.account.id(),
      txIncrementRequest
    );
    await client.submitTransaction(txResult);
    await window.helpers.waitForTransaction(
      txResult.executedTransaction().id().toHex()
    );

    // Create transaction with network note
    assembler = window.TransactionKernel.assembler()
      .withDebugMode(true)
      .withLibrary(accountComponentLib);

    let compiledNoteScript = await assembler.compileNoteScript(scriptCode);

    let noteInputs = new window.NoteInputs(new window.FeltArray([]));

    const randomInts = Array.from({ length: 4 }, () =>
      Math.floor(Math.random() * 100000)
    );
    let serialNum = new window.Word(new BigUint64Array(randomInts.map(BigInt)));
    let noteRecipient = new window.NoteRecipient(
      serialNum,
      compiledNoteScript,
      noteInputs
    );

    let noteAssets = new window.NoteAssets([]);

    let noteMetadata = new window.NoteMetadata(
      nativeAccount.id(),
      window.NoteType.Public,
      window.NoteTag.fromAccountId(accountBuilderResult.account.id()),
      window.NoteExecutionHint.none(),
      undefined
    );

    let note = new window.Note(noteAssets, noteMetadata, noteRecipient);

    let transactionRequest = new window.TransactionRequestBuilder()
      .withOwnOutputNotes(
        new window.OutputNotesArray([window.OutputNote.full(note)])
      )
      .build();

    let transactionResult = await client.newTransaction(
      nativeAccount.id(),
      transactionRequest
    );

    await client.submitTransaction(transactionResult);
    await window.helpers.waitForTransaction(
      transactionResult.executedTransaction().id().toHex()
    );

    // Wait for network account to update
    await window.helpers.waitForBlocks(2);

    let account = await client.getAccount(accountBuilderResult.account.id());
    let counter = account?.storage().getItem(0)?.toHex();

    return counter?.replace(/^0x/, "").replace(/^0+|0+$/g, "");
  });
};

test.describe("counter account component tests", () => {
  test("counter account component transaction completes successfully", async ({
    page,
  }) => {
    let finalCounter = await counterAccountComponent(page);
    expect(finalCounter).toEqual("2");
  });
});
