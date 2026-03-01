//! PVM execution engine (Appendix A of the Gray Paper).
//!
//! The PVM state machine: Ψ and Ψ₁ (single-step transition).

use crate::instruction::Opcode;
use crate::memory::Memory;
use grey_types::constants::PVM_REGISTER_COUNT;
use grey_types::Gas;
use thiserror::Error;

/// Exit reason for PVM execution (ε values).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExitReason {
    /// ∎: Normal halt.
    Halt,
    /// ☇: Panic / unexpected termination.
    Panic,
    /// ∞: Out of gas.
    OutOfGas,
    /// ×: Page fault at the given address.
    PageFault(u32),
    /// h̵×h: Host-call with the given identifier.
    HostCall(u32),
}

/// PVM instance state.
#[derive(Clone, Debug)]
pub struct Pvm {
    /// ϱ: Gas counter (remaining gas).
    pub gas: Gas,

    /// φ: 13 general-purpose 64-bit registers.
    pub registers: [u64; PVM_REGISTER_COUNT],

    /// µ: Memory state.
    pub memory: Memory,

    /// ı: Instruction counter (program counter).
    pub pc: u32,

    /// ζ: Instruction bytecode.
    pub code: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum PvmError {
    #[error("invalid opcode: {0}")]
    InvalidOpcode(u8),

    #[error("program counter out of bounds: {0}")]
    PcOutOfBounds(u32),
}

impl Pvm {
    /// Create a new PVM instance with the given code, registers, memory, and gas limit.
    pub fn new(code: Vec<u8>, registers: [u64; PVM_REGISTER_COUNT], memory: Memory, gas: Gas) -> Self {
        Self {
            gas,
            registers,
            memory,
            pc: 0,
            code,
        }
    }

    /// Execute a single instruction step Ψ₁.
    ///
    /// Returns the exit reason if the machine should stop, or Ok(()) to continue.
    pub fn step(&mut self) -> Result<Option<ExitReason>, PvmError> {
        // Check gas
        if self.gas == 0 {
            return Ok(Some(ExitReason::OutOfGas));
        }

        // Fetch opcode
        if self.pc as usize >= self.code.len() {
            return Ok(Some(ExitReason::Panic));
        }

        let opcode_byte = self.code[self.pc as usize];
        let opcode = match Opcode::from_byte(opcode_byte) {
            Some(op) => op,
            None => {
                // Unknown opcode: treat as trap
                return Ok(Some(ExitReason::Panic));
            }
        };

        // Deduct gas
        let cost = opcode.gas_cost();
        if self.gas < cost {
            return Ok(Some(ExitReason::OutOfGas));
        }
        self.gas -= cost;

        // Execute based on opcode category
        match opcode {
            Opcode::Trap => {
                return Ok(Some(ExitReason::Panic));
            }
            Opcode::Fallthrough => {
                // Advance to next instruction (skip based on instruction length)
                self.pc += 1;
            }
            Opcode::Ecalli => {
                // Host-call: read the immediate argument
                if self.pc as usize + 1 >= self.code.len() {
                    return Ok(Some(ExitReason::Panic));
                }
                // Decode variable-length immediate for the host-call number
                let (host_call_id, len) = self.decode_immediate_u32(self.pc as usize + 1);
                self.pc += 1 + len as u32;
                return Ok(Some(ExitReason::HostCall(host_call_id)));
            }
            _ => {
                // For now, all other instructions advance PC by the instruction length.
                // Full execution logic will be implemented per-opcode.
                let len = self.instruction_length(opcode_byte);
                self.pc += len as u32;
            }
        }

        Ok(None)
    }

    /// Run the machine until it exits.
    ///
    /// Returns (exit_reason, gas_used).
    pub fn run(&mut self) -> (ExitReason, Gas) {
        let initial_gas = self.gas;

        loop {
            match self.step() {
                Ok(Some(exit)) => {
                    let gas_used = initial_gas - self.gas;
                    return (exit, gas_used);
                }
                Ok(None) => continue,
                Err(_) => {
                    let gas_used = initial_gas - self.gas;
                    return (ExitReason::Panic, gas_used);
                }
            }
        }
    }

    /// Determine instruction length from the opcode byte.
    /// This is a simplified version; the full implementation must handle
    /// variable-length immediates per Appendix A.4.
    fn instruction_length(&self, opcode: u8) -> usize {
        match opcode {
            // No arguments
            0 | 17 => 1,
            // One immediate
            78 => {
                // ecalli: 1 byte opcode + variable immediate
                let (_, len) = self.decode_immediate_u32(self.pc as usize + 1);
                1 + len
            }
            // Most other instructions have variable length.
            // The general encoding uses the `ℓ` length field from the basic block table.
            // For now, default to minimal length.
            _ => {
                // Compute from the code's skip table if available.
                // Fallback: 2 bytes (opcode + register byte)
                2
            }
        }
    }

    /// Decode a variable-length unsigned immediate from the code.
    fn decode_immediate_u32(&self, offset: usize) -> (u32, usize) {
        if offset >= self.code.len() {
            return (0, 0);
        }

        // Simple: treat remaining bytes of the instruction as LE immediate.
        // Full implementation reads based on `ℓ - fixed_args`.
        let mut value: u32 = 0;
        let mut bytes_read = 0;
        for i in 0..4 {
            if offset + i >= self.code.len() {
                break;
            }
            value |= (self.code[offset + i] as u32) << (i * 8);
            bytes_read += 1;
        }
        (value, bytes_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    #[test]
    fn test_trap_instruction() {
        let code = vec![0]; // trap
        let mut vm = Pvm::new(code, [0; 13], Memory::new(), 100);
        let (exit, _gas) = vm.run();
        assert_eq!(exit, ExitReason::Panic);
    }

    #[test]
    fn test_out_of_gas() {
        let code = vec![17; 1000]; // many fallthrough instructions
        let mut vm = Pvm::new(code, [0; 13], Memory::new(), 5);
        let (exit, gas_used) = vm.run();
        assert_eq!(exit, ExitReason::OutOfGas);
        assert_eq!(gas_used, 5);
    }

    #[test]
    fn test_empty_program() {
        let code = vec![];
        let mut vm = Pvm::new(code, [0; 13], Memory::new(), 100);
        let (exit, _) = vm.run();
        assert_eq!(exit, ExitReason::Panic);
    }
}
