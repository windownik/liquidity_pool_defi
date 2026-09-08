import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MyCounter } from "../target/types/my_counter";
// @ts-ignore
import fs from "fs";
// @ts-ignore
import os from "os";

async function main() {
// 1. Подключаемся к Devnet
    const connection = new anchor.web3.Connection("https://api.devnet.solana.com", "confirmed");

    // 2. Загружаем ваш локальный кошелек из ~/.config/solana/id.json
    const keypairPath = `${os.homedir()}/.config/solana/id.json`;
    const secretKey = Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath, "utf8")));
    const wallet = new anchor.Wallet(anchor.web3.Keypair.fromSecretKey(secretKey));

    // 3. Создаем провайдер и устанавливаем его
    const provider = new anchor.AnchorProvider(connection, wallet, {
        preflightCommitment: "confirmed",
    });
    anchor.setProvider(provider);

    // 4. Инициализируем программу
    const program = anchor.workspace.MyCounter as Program<MyCounter>;
    console.log("Program ID:", program.programId.toBase58());
    console.log("Payer / Authority:", provider.wallet.publicKey.toBase58());

    // 3. Вычисляем PDA для аккаунта счётчика (seed = "state")
    const [counterPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("counter")],
        program.programId
    );

    console.log("Counter PDA:", counterPda.toBase58());

    // ----------------------------------------------------------------
    // ШАГ 1: Инициализация счётчика (если он ещё не создан)
    // ----------------------------------------------------------------
    try {
        console.log("\nОтправка транзакции initialize...");
        const txInit = await program.methods
            .initialize()
            .accounts({
                payer: provider.wallet.publicKey,
                // Anchor автоматически подставит state, systemProgram и bump,
                // но явное указание помогает избежать неоднозначностей.
            })
            .rpc();

        console.log("Initialize TX Signature:", txInit);
        console.log(`Explorer: https://explorer.solana.com/tx/${txInit}?cluster=devnet`);
    } catch (err: any) {
        if (err.toString().includes("already in use")) {
            console.log("Аккаунт счётчика уже инициализирован, пропускаем initialize.");
        } else {
            console.error("Ошибка при initialize:", err);
        }
    }

    // ----------------------------------------------------------------
    // ШАГ 2: Вызов инструкции increment
    // ----------------------------------------------------------------
    console.log("\nОтправка транзакции increment...");
    try {
        const txInc = await program.methods
            .increment()
            .accounts({
                authority: provider.wallet.publicKey,
            })
            .rpc();

        console.log("Increment TX Signature:", txInc);
        console.log(`Explorer: https://explorer.solana.com/tx/${txInc}?cluster=devnet`);
    } catch (err) {
        console.error("Ошибка при increment:", err);
    }

    // ----------------------------------------------------------------
    // ШАГ 3: Чтение состояния аккаунта из блокчейна
    // ----------------------------------------------------------------
    console.log("\nСчитывание текущего состояния из Devnet...");
    const counterAccount = await program.account.counter.fetch(counterPda);

    console.log("---------------------------------------");
    console.log("Текущий count:", counterAccount.count.toString());
    console.log("Владелец (authority):", counterAccount.authority.toBase58());
    console.log("---------------------------------------");
}

main().then(
    () => process.exit(0),
    (err) => {
        console.error(err);
        process.exit(1);
    }
);