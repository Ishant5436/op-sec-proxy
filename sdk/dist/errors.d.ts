/**
 * OP Security Proxy Revert Decoders & Error Classes
 */
import { JsonRpcErrorObject } from "./types";
export declare class RevertBlockedError extends Error {
    readonly code: number;
    readonly rawRevertData?: string;
    readonly decodedReason?: string;
    readonly estimatedGasSaved?: number;
    readonly latencyMs?: number;
    constructor(rpcError: JsonRpcErrorObject);
}
/**
 * Decodes standard ABI error strings: Error(string) selector 0x08c379a0 and Panic(uint256) 0x4e487b71
 */
export declare function decodeStandardRevert(hexData: string): string | null;
