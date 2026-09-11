use alloy::primitives::U256;
use alloy::sol_types::{SolError, sol};

sol! {
    /// Standard ABI-encoded error string: Error(string) -> 0x08c379a0
    error Error(string message);

    /// Solidity 0.8+ Panic error: Panic(uint256) -> 0x4e487b71
    error Panic(uint256 code);

    /// ERC-6093: Custom error for ERC-20 insufficient balance (OpenZeppelin v5) -> 0xe450d38c
    error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);

    /// ERC-6093: Custom error for ERC-20 insufficient allowance (OpenZeppelin v5) -> 0xfb8f41b2
    error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);

    /// ERC-6093: Custom error for ERC-20 invalid approver -> 0xe602df05
    error ERC20InvalidApprover(address approver);

    /// ERC-6093: Custom error for ERC-20 invalid receiver -> 0xec447034
    error ERC20InvalidReceiver(address receiver);

    /// ERC-6093: Custom error for ERC-20 invalid sender -> 0x96c6fd1e
    error ERC20InvalidSender(address sender);

    /// ERC-6093: Custom error for ERC-20 invalid spender -> 0x3b4da69f
    error ERC20InvalidSpender(address spender);

    /// ERC-20: Caller lacks sufficient balance (generic alias)
    error InsufficientBalance(address account, uint256 currentBalance, uint256 requiredBalance);

    /// ERC-20: Spender lacks sufficient allowance (generic alias)
    error InsufficientAllowance(address spender, uint256 currentAllowance, uint256 requiredAllowance);

    /// ERC-20 / ERC-721: Operation on or from zero address
    error ZeroAddress();

    /// Token transfer failed (e.g. SafeERC20 check)
    error TransferFailed();

    /// DEX: Minimum output amount not satisfied by execution
    error SlippageExceeded(uint256 expected, uint256 actual);

    /// DEX: Transaction executed after specified deadline timestamp
    error DeadlineExpired(uint256 deadline, uint256 currentTimestamp);

    /// Access control: Caller not authorized
    error Unauthorized();
}

/// Structured decode result from raw revert bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevertDecodeResult {
    pub decoded_reason: Option<String>,
    pub selector_hex: Option<String>,
    pub error_name: Option<String>,
}

/// Returns a human-readable description for canonical Solidity panic codes.
pub fn format_panic_reason(code: U256) -> String {
    let code_u64 = if code <= U256::from(0xFF) {
        code.to::<u64>()
    } else {
        0xFFFF
    };

    let desc = match code_u64 {
        0x01 => "Assert failed",
        0x11 => "Arithmetic overflow / underflow",
        0x12 => "Division by zero",
        0x21 => "Invalid enum value conversion",
        0x22 => "Storage byte array encoding error",
        0x31 => "Empty array pop",
        0x32 => "Array index out of bounds",
        0x41 => "Allocation of too much memory",
        0x51 => "Zero initialized internal function pointer",
        _ => "Unrecognized panic code",
    };
    format!("Panic: {} (0x{:02x})", desc, code_u64)
}

/// Decodes standard and custom Solidity revert payloads.
/// Employs compile-time SolError signatures with safe ABI decoding.
pub fn decode_revert_output(output: &[u8]) -> RevertDecodeResult {
    if output.len() < 4 {
        return RevertDecodeResult {
            decoded_reason: None,
            selector_hex: None,
            error_name: None,
        };
    }

    let selector: [u8; 4] = [output[0], output[1], output[2], output[3]];
    let selector_hex = format!("0x{:08x}", u32::from_be_bytes(selector));

    match selector {
        Error::SELECTOR => decode_string_error(output, selector_hex),
        Panic::SELECTOR => decode_panic_error(output, selector_hex),
        ERC20InsufficientBalance::SELECTOR => {
            decode_erc20_insufficient_balance(output, selector_hex)
        }
        ERC20InsufficientAllowance::SELECTOR => {
            decode_erc20_insufficient_allowance(output, selector_hex)
        }
        ERC20InvalidApprover::SELECTOR => RevertDecodeResult {
            decoded_reason: Some("ERC-6093: Invalid approver address (zero address)".to_string()),
            selector_hex: Some(selector_hex),
            error_name: Some("ERC20InvalidApprover".to_string()),
        },
        ERC20InvalidReceiver::SELECTOR => RevertDecodeResult {
            decoded_reason: Some("ERC-6093: Invalid receiver address (zero address)".to_string()),
            selector_hex: Some(selector_hex),
            error_name: Some("ERC20InvalidReceiver".to_string()),
        },
        ERC20InvalidSender::SELECTOR => RevertDecodeResult {
            decoded_reason: Some("ERC-6093: Invalid sender address (zero address)".to_string()),
            selector_hex: Some(selector_hex),
            error_name: Some("ERC20InvalidSender".to_string()),
        },
        ERC20InvalidSpender::SELECTOR => RevertDecodeResult {
            decoded_reason: Some("ERC-6093: Invalid spender address (zero address)".to_string()),
            selector_hex: Some(selector_hex),
            error_name: Some("ERC20InvalidSpender".to_string()),
        },
        InsufficientBalance::SELECTOR => decode_insufficient_balance(output, selector_hex),
        InsufficientAllowance::SELECTOR => decode_insufficient_allowance(output, selector_hex),
        ZeroAddress::SELECTOR => RevertDecodeResult {
            decoded_reason: Some("Operation rejected: Zero address specified".to_string()),
            selector_hex: Some(selector_hex),
            error_name: Some("ZeroAddress".to_string()),
        },
        TransferFailed::SELECTOR => RevertDecodeResult {
            decoded_reason: Some("Token transfer failed".to_string()),
            selector_hex: Some(selector_hex),
            error_name: Some("TransferFailed".to_string()),
        },
        SlippageExceeded::SELECTOR => decode_slippage_exceeded(output, selector_hex),
        DeadlineExpired::SELECTOR => decode_deadline_expired(output, selector_hex),
        Unauthorized::SELECTOR => RevertDecodeResult {
            decoded_reason: Some("Access denied: Unauthorized caller".to_string()),
            selector_hex: Some(selector_hex),
            error_name: Some("Unauthorized".to_string()),
        },
        _ => RevertDecodeResult {
            decoded_reason: None,
            selector_hex: Some(selector_hex),
            error_name: None,
        },
    }
}

fn decode_string_error(output: &[u8], selector_hex: String) -> RevertDecodeResult {
    match Error::abi_decode(output) {
        Ok(err) => RevertDecodeResult {
            decoded_reason: Some(err.message),
            selector_hex: Some(selector_hex),
            error_name: Some("Error".to_string()),
        },
        Err(_) => RevertDecodeResult {
            decoded_reason: None,
            selector_hex: Some(selector_hex),
            error_name: Some("Error".to_string()),
        },
    }
}

fn decode_panic_error(output: &[u8], selector_hex: String) -> RevertDecodeResult {
    match Panic::abi_decode(output) {
        Ok(err) => RevertDecodeResult {
            decoded_reason: Some(format_panic_reason(err.code)),
            selector_hex: Some(selector_hex),
            error_name: Some("Panic".to_string()),
        },
        Err(_) => RevertDecodeResult {
            decoded_reason: None,
            selector_hex: Some(selector_hex),
            error_name: Some("Panic".to_string()),
        },
    }
}

fn decode_erc20_insufficient_balance(output: &[u8], selector_hex: String) -> RevertDecodeResult {
    match ERC20InsufficientBalance::abi_decode(output) {
        Ok(err) => RevertDecodeResult {
            decoded_reason: Some(format!(
                "ERC-6093 InsufficientBalance: sender {} has balance {} (needed: {})",
                err.sender, err.balance, err.needed
            )),
            selector_hex: Some(selector_hex),
            error_name: Some("ERC20InsufficientBalance".to_string()),
        },
        Err(_) => RevertDecodeResult {
            decoded_reason: None,
            selector_hex: Some(selector_hex),
            error_name: Some("ERC20InsufficientBalance".to_string()),
        },
    }
}

fn decode_erc20_insufficient_allowance(output: &[u8], selector_hex: String) -> RevertDecodeResult {
    match ERC20InsufficientAllowance::abi_decode(output) {
        Ok(err) => RevertDecodeResult {
            decoded_reason: Some(format!(
                "ERC-6093 InsufficientAllowance: spender {} has allowance {} (needed: {})",
                err.spender, err.allowance, err.needed
            )),
            selector_hex: Some(selector_hex),
            error_name: Some("ERC20InsufficientAllowance".to_string()),
        },
        Err(_) => RevertDecodeResult {
            decoded_reason: None,
            selector_hex: Some(selector_hex),
            error_name: Some("ERC20InsufficientAllowance".to_string()),
        },
    }
}

fn decode_insufficient_balance(output: &[u8], selector_hex: String) -> RevertDecodeResult {
    match InsufficientBalance::abi_decode(output) {
        Ok(err) => RevertDecodeResult {
            decoded_reason: Some(format!(
                "ERC-20 InsufficientBalance: account {} has balance {} (required: {})",
                err.account, err.currentBalance, err.requiredBalance
            )),
            selector_hex: Some(selector_hex),
            error_name: Some("InsufficientBalance".to_string()),
        },
        Err(_) => RevertDecodeResult {
            decoded_reason: None,
            selector_hex: Some(selector_hex),
            error_name: Some("InsufficientBalance".to_string()),
        },
    }
}

fn decode_insufficient_allowance(output: &[u8], selector_hex: String) -> RevertDecodeResult {
    match InsufficientAllowance::abi_decode(output) {
        Ok(err) => RevertDecodeResult {
            decoded_reason: Some(format!(
                "ERC-20 InsufficientAllowance: spender {} has allowance {} (required: {})",
                err.spender, err.currentAllowance, err.requiredAllowance
            )),
            selector_hex: Some(selector_hex),
            error_name: Some("InsufficientAllowance".to_string()),
        },
        Err(_) => RevertDecodeResult {
            decoded_reason: None,
            selector_hex: Some(selector_hex),
            error_name: Some("InsufficientAllowance".to_string()),
        },
    }
}

fn decode_slippage_exceeded(output: &[u8], selector_hex: String) -> RevertDecodeResult {
    match SlippageExceeded::abi_decode(output) {
        Ok(err) => RevertDecodeResult {
            decoded_reason: Some(format!(
                "DEX SlippageExceeded: minimum expected output {} but received {}",
                err.expected, err.actual
            )),
            selector_hex: Some(selector_hex),
            error_name: Some("SlippageExceeded".to_string()),
        },
        Err(_) => RevertDecodeResult {
            decoded_reason: None,
            selector_hex: Some(selector_hex),
            error_name: Some("SlippageExceeded".to_string()),
        },
    }
}

fn decode_deadline_expired(output: &[u8], selector_hex: String) -> RevertDecodeResult {
    match DeadlineExpired::abi_decode(output) {
        Ok(err) => RevertDecodeResult {
            decoded_reason: Some(format!(
                "DEX DeadlineExpired: deadline {} expired at timestamp {}",
                err.deadline, err.currentTimestamp
            )),
            selector_hex: Some(selector_hex),
            error_name: Some("DeadlineExpired".to_string()),
        },
        Err(_) => RevertDecodeResult {
            decoded_reason: None,
            selector_hex: Some(selector_hex),
            error_name: Some("DeadlineExpired".to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_canonical_selectors_match_known_hashes() {
        assert_eq!(Error::SELECTOR, [0x08, 0xc3, 0x79, 0xa0]);
        assert_eq!(Panic::SELECTOR, [0x4e, 0x48, 0x7b, 0x71]);
    }

    #[test]
    fn test_decode_standard_error_string() {
        let err = Error {
            message: "TRANSFER_AMOUNT_EXCEEDS_BALANCE".to_string(),
        };
        let encoded = err.abi_encode();
        let decoded = decode_revert_output(&encoded);

        assert_eq!(decoded.error_name, Some("Error".to_string()));
        assert_eq!(
            decoded.decoded_reason,
            Some("TRANSFER_AMOUNT_EXCEEDS_BALANCE".to_string())
        );
        assert_eq!(decoded.selector_hex, Some("0x08c379a0".to_string()));
    }

    #[test]
    fn test_decode_panic_arithmetic_overflow() {
        let panic = Panic {
            code: U256::from(0x11),
        };
        let encoded = panic.abi_encode();
        let decoded = decode_revert_output(&encoded);

        assert_eq!(decoded.error_name, Some("Panic".to_string()));
        assert!(
            decoded
                .decoded_reason
                .as_ref()
                .unwrap()
                .contains("Arithmetic overflow")
        );
        assert_eq!(decoded.selector_hex, Some("0x4e487b71".to_string()));
    }

    #[test]
    fn test_decode_insufficient_allowance() {
        let spender = address!("1111111111111111111111111111111111111111");
        let err = InsufficientAllowance {
            spender,
            currentAllowance: U256::from(100),
            requiredAllowance: U256::from(1000),
        };
        let encoded = err.abi_encode();
        let decoded = decode_revert_output(&encoded);

        assert_eq!(
            decoded.error_name,
            Some("InsufficientAllowance".to_string())
        );
        let reason = decoded.decoded_reason.unwrap();
        assert!(reason.contains("InsufficientAllowance"));
        assert!(reason.contains("1000"));
    }

    #[test]
    fn test_decode_slippage_exceeded() {
        let err = SlippageExceeded {
            expected: U256::from(5000),
            actual: U256::from(4800),
        };
        let encoded = err.abi_encode();
        let decoded = decode_revert_output(&encoded);

        assert_eq!(decoded.error_name, Some("SlippageExceeded".to_string()));
        let reason = decoded.decoded_reason.unwrap();
        assert!(reason.contains("minimum expected output 5000"));
        assert!(reason.contains("received 4800"));
    }

    #[test]
    fn test_unrecognized_selector_handling() {
        let random_calldata = vec![0x12, 0x34, 0x56, 0x78, 0x00, 0x01];
        let decoded = decode_revert_output(&random_calldata);

        assert_eq!(decoded.error_name, None);
        assert_eq!(decoded.decoded_reason, None);
        assert_eq!(decoded.selector_hex, Some("0x12345678".to_string()));
    }

    #[test]
    fn test_erc6093_selectors_and_decoding() {
        assert_eq!(ERC20InsufficientBalance::SELECTOR, [0xe4, 0x50, 0xd3, 0x8c]);
        assert_eq!(
            ERC20InsufficientAllowance::SELECTOR,
            [0xfb, 0x8f, 0x41, 0xb2]
        );

        let sender = address!("2222222222222222222222222222222222222222");
        let err = ERC20InsufficientBalance {
            sender,
            balance: U256::from(50),
            needed: U256::from(100),
        };
        let encoded = err.abi_encode();
        let decoded = decode_revert_output(&encoded);

        assert_eq!(
            decoded.error_name,
            Some("ERC20InsufficientBalance".to_string())
        );
        assert_eq!(decoded.selector_hex, Some("0xe450d38c".to_string()));
        let reason = decoded.decoded_reason.unwrap();
        assert!(reason.contains("ERC-6093 InsufficientBalance"));
        assert!(reason.contains("50"));
        assert!(reason.contains("100"));
    }

    #[test]
    fn test_truncated_calldata_returns_none() {
        let short_data = vec![0x08, 0xc3];
        let decoded = decode_revert_output(&short_data);

        assert_eq!(decoded.error_name, None);
        assert_eq!(decoded.decoded_reason, None);
        assert_eq!(decoded.selector_hex, None);
    }
}
