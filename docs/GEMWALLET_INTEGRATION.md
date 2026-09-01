# GemWallet Integration Guide: OP Security Proxy

This guide provides the exact integration blueprint for **GemWallet** (and other EIP-1193 mobile/browser wallets) to protect users from paying gas for failed transactions on Optimism and the Superchain.

---

## 1. Overview & Architecture

When connected through **OP Security Proxy**, outbound `eth_sendRawTransaction` and `eth_sendTransaction` requests are simulated locally via `revm` before reaching the network mempool.

```
┌────────────────────────────────────────────────────────┐
│                   GemWallet Frontend                   │
│                                                        │
│  User initiates Swap / Transfer ──► Signs Transaction │
└──────────────────────────┬─────────────────────────────┘
                           │ eth_sendRawTransaction
                           ▼
┌────────────────────────────────────────────────────────┐
│                   OP Security Proxy                    │
│                                                        │
│  1. Forks OP L2 State locally in revm (<50ms)          │
│  2. Executes transaction in zero-trust sandbox         │
│  3. If Reverted: Intercepts & returns JSON-RPC -32000  │
└──────────────────────────┬─────────────────────────────┘
                           │
       ┌───────────────────┴───────────────────┐
       │ [Simulation Fails]                    │ [Simulation Passes]
       ▼                                       ▼
┌──────────────────────────────┐    ┌──────────────────────────────┐
│ Return Revert Error Payload  │    │ Broadcast to OP Sequencer    │
│ • decoded_reason             │    │ • Real Transaction Hash      │
│ • estimated_gas_saved        │    │ • Confirmed On-Chain Receipt │
└──────────────┬───────────────┘    └──────────────────────────────┘
               │
               ▼
┌────────────────────────────────────────────────────────┐
│                   GemWallet Frontend                   │
│                                                        │
│  Renders UI Banner:                                    │
│  "⚠️ Transaction Blocked: Insufficient Allowance"       │
│  "🛡️ Protected by OP Security: $0.42 gas saved"         │
└────────────────────────────────────────────────────────┘
```

---

## 2. Standard JSON-RPC Error Payload

When a transaction would revert on-chain, the proxy intercepts the RPC call and returns an explicit error payload:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "Execution reverted: Insufficient allowance",
    "data": {
      "revert_data": "0x08c379a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000016496e73756666696369656e7420616c6c6f77616e636500000000000000000000",
      "decoded_reason": "Insufficient allowance",
      "estimated_gas_saved": 210000,
      "simulation_latency_ms": 12
    }
  }
}
```

---

## 3. Client Integration (TypeScript / React Native)

Using the official `@op-sec/client` SDK:

```typescript
import { OpSecClient, RevertBlockedError } from "@op-sec/client";

// 1. Initialize client pointing to OP Security Proxy
const opSec = new OpSecClient({
  proxyUrl: "https://proxy.op-security.org", // Or local node: http://localhost:8545
  fallbackRpcUrl: "https://mainnet.optimism.io"
});

// 2. Wrap transaction broadcast
async function sendTransactionWithProtection(signedTxHex: string) {
  try {
    const txHash = await opSec.request<string>({
      jsonrpc: "2.0",
      id: Date.now(),
      method: "eth_sendRawTransaction",
      params: [signedTxHex]
    });

    console.log("Transaction successfully broadcasted:", txHash);
    return { success: true, txHash };
  } catch (error) {
    if (error instanceof RevertBlockedError) {
      // User lost $0 in gas fees!
      console.warn("Preemptively Blocked Revert:", error.decodedReason);
      console.info("Estimated Gas Saved (Wei):", error.estimatedGasSaved);

      // Render User-Facing Alert in GemWallet
      displayGasProtectionBanner({
        title: "Transaction Safely Intercepted",
        reason: error.decodedReason || "Transaction would have failed on-chain",
        gasSaved: error.estimatedGasSaved
      });

      return { success: false, blocked: true, reason: error.decodedReason };
    }

    // Standard RPC or network error
    throw error;
  }
}
```

---

## 4. Drop-In EIP-1193 Provider Hook

For wallets using standard Web3 provider providers (`window.ethereum` or mobile Web3 webviews):

```typescript
import { decodeStandardRevert } from "@op-sec/client";

export function handleWalletRpcError(error: any) {
  if (error?.code === -32000 && error?.data?.decoded_reason) {
    return {
      isRevert: true,
      message: `Transaction stopped: ${error.data.decoded_reason}`,
      gasSavedWei: error.data.estimated_gas_saved || 0
    };
  }
  
  if (typeof error?.data === "string") {
    const decoded = decodeStandardRevert(error.data);
    if (decoded) {
      return {
        isRevert: true,
        message: `Transaction stopped: ${decoded}`,
        gasSavedWei: 0
      };
    }
  }

  return { isRevert: false, message: error.message };
}
```

---

## 5. Support & Security Invariants
* **Zero Private Key Exposure:** The proxy only handles signed raw bytes (`eth_sendRawTransaction`); it never touches private keys.
* **Open Source:** Permissive MIT License.
* **Repository:** [https://github.com/Ishant5436/op-sec-proxy](https://github.com/Ishant5436/op-sec-proxy)
