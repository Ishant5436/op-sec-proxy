/**
 * OP Security Proxy Revert Decoders & Error Classes
 */

import { JsonRpcErrorObject } from "./types";

export class RevertBlockedError extends Error {
  public readonly code: number;
  public readonly rawRevertData?: string;
  public readonly decodedReason?: string;
  public readonly estimatedGasSaved?: number;
  public readonly latencyMs?: number;

  constructor(rpcError: JsonRpcErrorObject) {
    let reason = rpcError.message;
    let rawData: string | undefined;
    let gasSaved: number | undefined;
    let latency: number | undefined;

    if (typeof rpcError.data === "object" && rpcError.data !== null) {
      if (rpcError.data.decoded_reason) {
        reason = rpcError.data.decoded_reason;
      }
      rawData = rpcError.data.revert_data;
      gasSaved = rpcError.data.estimated_gas_saved;
      latency = rpcError.data.simulation_latency_ms;
    } else if (typeof rpcError.data === "string") {
      rawData = rpcError.data;
      const decoded = decodeStandardRevert(rpcError.data);
      if (decoded) reason = decoded;
    }

    super(`[OP Security Proxy] Transaction Reverted: ${reason}`);
    this.name = "RevertBlockedError";
    this.code = rpcError.code;
    this.decodedReason = reason;
    this.rawRevertData = rawData;
    this.estimatedGasSaved = gasSaved;
    this.latencyMs = latency;
  }
}

/**
 * Decodes standard ABI error strings: Error(string) selector 0x08c379a0 and Panic(uint256) 0x4e487b71
 */
export function decodeStandardRevert(hexData: string): string | null {
  if (!hexData || !hexData.startsWith("0x")) return null;
  const raw = hexData.slice(2);

  // 1. Error(string) selector: 08c379a0
  if (raw.startsWith("08c379a0") && raw.length >= 136) {
    try {
      const lengthOffset = 8 + 64;
      const stringLengthHex = raw.slice(lengthOffset, lengthOffset + 64);
      const stringLength = parseInt(stringLengthHex, 16);
      const dataOffset = lengthOffset + 64;
      const textHex = raw.slice(dataOffset, dataOffset + stringLength * 2);
      
      let str = "";
      for (let i = 0; i < textHex.length; i += 2) {
        str += String.fromCharCode(parseInt(textHex.slice(i, i + 2), 16));
      }
      return str;
    } catch {
      return "Execution reverted with custom Error(string)";
    }
  }

  // 2. Panic(uint256) selector: 4e487b71
  if (raw.startsWith("4e487b71") && raw.length >= 72) {
    const codeHex = raw.slice(8, 72);
    const code = parseInt(codeHex, 16);
    const panicCodes: Record<number, string> = {
      0x01: "Assert failed",
      0x11: "Arithmetic overflow / underflow",
      0x12: "Division by zero",
      0x21: "Invalid enum value conversion",
      0x22: "Storage byte array encoding error",
      0x31: "Empty array pop",
      0x32: "Array index out of bounds",
      0x41: "Allocation of too much memory",
      0x51: "Zero initialized internal function pointer"
    };
    return panicCodes[code] || `Panic error code 0x${code.toString(16)}`;
  }

  return null;
}
