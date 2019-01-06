//
// genetic - genetic programming experiments
// Copyright (c) 2019 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//
//
// File description:
//   Module: virtual machine.
//

/// Virtual machine's computational data type (type of the `reg_v`'s value).
pub type RegValue = f32;

/// Virtual machine's state.
#[derive(Clone)]
pub struct VmState {
    /// Data slots.
    pub data: Vec<RegValue>,
    /// Index register.
    pub reg_i: i32,
    /// Value register.
    pub reg_v: RegValue,
    /// Current instruction pointer.
    pub iptr: usize
}

impl VmState {
    pub fn reset(&mut self) {
        self.data = vec![0.0; self.data.len()];
        self.reg_i = 0;
        self.reg_v = 0.0;
        self.iptr = 0;
    }
}

///
/// Virtual machine instruction opcodes.
///
/// Instruction set is based on Slash/A language by Artur B Adib.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpCode {
    /// Assign value to `reg_i`.
    SetI(i32),
    /// Read value from the specified input to `reg_v`.
    Input(i32),
    /// Write `reg_v` to the specified output.
    Output(i32),
    /// Assign `reg_i` to `reg_v`.
    ItoV,
    /// Assign `reg_v` to `reg_i`.
    VtoI,
    /// Increment `reg_v`.
    IncV,
    /// Decrement `reg_v`.
    DecV,
    /// Increment `reg_i`.
    IncI,
    /// Decrement `reg_i`.
    DecI,
    /// Assign `data[reg_i]` to `reg_v`.
    Load,
    /// Assign `reg_v` to `data[reg_i]`.
    Store,
    /// Swap `reg_v` and `data[reg_i]`.
    Swap,
    /// Set jump location for the `GotoIfP` on the same nesting level.
    EndGoTo,
    /// If `reg_v` >= 0, jump backward to the corresponding `EndGoTo`.
    GoToIfP,
    /// If `reg_v` < 0, jump forward to the corresponding `EndJump`.
    JumpIfN,
    /// Set jump location for the `JumpIfN` on the same nesting level.
    EndJump,
    /// If `reg_v` < 0, skip the next instruction.
    IfP,
    /// If `reg_v` >= 0, skip the next instruction.
    IfN,
    /// Compares `reg_v` with `data[reg_i]` and sets `reg_v` to:
    /// * 0 if equal
    /// * -1 if less than
    /// * 1 if greater than
    Cmp,
    /// Add `data[reg_i]` to `reg_v`.
    Add,
    /// Subtract `data[reg_i]` from `reg_v`.
    Sub,
    /// Multiply `reg_v` by `data[reg_i]`.
    Mul,
    /// Divide `reg_v` by `data[reg_i]` if non-zero, otherwise do nothing.
    Div,
    /// Set `reg_v` to its absolute value.
    Abs,
    /// Flip sign of `reg_v`.
    Neg,
    /// Set `reg_v` to its square root if non-negative, otherwise set to zero.
    Sqrt,
    ///Do nothing.
    Nop
}

/// Handler of `OpCode::Input` and `OpCode::Output`.
pub trait InputOutputHandler {
    fn input(&mut self, input_num: i32) -> RegValue;
    fn output(&mut self, output_num: i32, output_val: RegValue);
    fn check_end_condition(&self, num_execd_instructions: usize) -> bool;
}

/// Reason for ending virtual machine program execution.
#[derive(Debug, PartialEq)]
pub enum EndReason {
    LastInstructionReached,
    NumExecInstructions,
    EndConditionMet
}

impl std::fmt::Display for EndReason {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Program that runs on virtual machine.
#[derive(Clone)]
pub struct Program {
    /// Instructions.
    instr: Vec<OpCode>,

    /// Number of virtual machine data slots used by program.
    num_data_slots: usize,

    ///
    /// Contains destination and source addresses (indices in `instr`).
    ///
    /// Each element corresponds to the same-index element in `instr`.
    /// Only those corresponding to `GoToIfP`, `EndGoTo`, `JumpIfN`, `EndJump` are `Some(…)`.
    ///
    jump_table: Vec<Option<usize>>,

    /// If true, crossing of `GoToIfP`/`EndGoTo` pairs by `JumpIfN`/`EndJump`
    /// pairs and vice versa is allowed. Otherwise, instructions that would cause crossing are disabled
    /// in the jump table.
    allow_crossing_blocks: bool
}

impl Program {
    ///
    /// Creates new program.
    ///
    /// # Parameters
    ///
    /// * `instruction` - Instruction list.
    /// * `num_data_slots` - Number of virtual machine data slots used by program.
    /// * `allow_crossing_blocks` - If true, crossing of `GoToIfP`/`EndGoTo` pairs by `JumpIfN`/`EndJump`
    /// pairs and vice versa is allowed. Otherwise, instructions that would cause crossing are disabled
    /// in the jump table.
    ///
    pub fn new(instructions: &[OpCode], num_data_slots: usize, allow_crossing_blocks: bool) -> Program {
        let mut jump_table = Program::create_jump_table(instructions);
        if !allow_crossing_blocks {
            Program::deactivate_crossing_blocks(instructions, &mut jump_table);
        }
        Program{
            instr: instructions.to_vec(),
            num_data_slots,
            jump_table,
            allow_crossing_blocks
        }
    }

    pub fn get_instr(&self) -> &[OpCode] {
        &self.instr
    }

    pub fn get_num_data_slots(&self) -> usize {
        self.num_data_slots
    }

    ///
    /// Returns program's jump table.
    ///
    /// Jump table contains destination and source addresses = indices in program's instruction list.
    /// Each element corresponds to the same-index element in program's instruction list.
    /// Only those corresponding to `GoToIfP`, `EndGoTo`, `JumpIfN`, `EndJump` are `Some(…)`.
    ///
    pub fn get_jump_table(&self) -> &[Option<usize>] {
        &self.jump_table
    }

    ///
    /// Creates a jump table.
    ///
    /// For each control flow instruction in `instr` (except `IfP`, `IfN`) the corresponding element
    /// of the result is a source or destination address (an index in `instr`).
    /// Nesting of `GoToIfP`/`EndGoTo` & `JumpIfN`/`EndJump` pairs is taken into account.
    ///
    fn create_jump_table(instr: &[OpCode]) -> Vec<Option<usize>> {
        let mut stack_end_goto: Vec<usize> = vec![];  // positions of the most recently encountered `OpCode::EndGoto`
        let mut stack_jump: Vec<usize> = vec![];  // positions of the most recently encountered `OpCode::JumpIfN`

        let mut jump_table: Vec<Option<usize>> = vec![None; instr.len()];

        for i in 0..instr.len() {
            match instr[i] {
                OpCode::EndGoTo => stack_end_goto.push(i),
                OpCode::JumpIfN => stack_jump.push(i),
                OpCode::GoToIfP => if !stack_end_goto.is_empty() {
                    let back = stack_end_goto.pop().unwrap();
                    jump_table[i] = Some(back);
                    jump_table[back] = Some(i);
                },
                OpCode::EndJump => if !stack_jump.is_empty() {
                    let back = stack_jump.pop().unwrap();
                    jump_table[back] = Some(i);
                    jump_table[i] = Some(back);
                },
                _ => ()
            }
        }

        jump_table
    }

    ///
    /// Modifies the specified jump table to deactivate any `GoToIfP`/`EndGoTo` pairs
    /// that cross `JumpIfN`/`EndJump` pairs and vice versa.
    ///
    fn deactivate_crossing_blocks(instr: &[OpCode], jump_table: &mut Vec<Option<usize>>) {
        let mut open_blocks: Vec<usize> = vec![];

        for pos in 0..instr.len() {
            match instr[pos] {
                OpCode::EndGoTo | OpCode::JumpIfN => if jump_table[pos].is_some() {
                    open_blocks.push(pos);
                },
                OpCode::GoToIfP | OpCode::EndJump => if jump_table[pos].is_some() {
                    loop {
                        let last = open_blocks.pop().unwrap();
                        // a block ends here; going towards its beginning, deactivate any other open blocks
                        if last != jump_table[pos].unwrap() {
                            let blk_start = last;
                            let blk_end = jump_table[last].unwrap();
                            jump_table[blk_start] = None;
                            jump_table[blk_end] = None;
                        } else {
                            break;
                        }
                    }
                },
                _ => ()
            }
        }
    }

    ///
    /// Returns an optimized version of the program: sequences of instructions without effect are removed.
    ///
    /// See the `optimization_tests` module in this file for examples.
    ///
    pub fn get_optimized(&self) -> Program {
        let mut opt_instr: Vec<OpCode> = vec![]; // optimized instruction list (in reverse)

        if self.instr.len() < 2 { return self.clone(); }

        // scan `self.instr` backwards and look for removable sequences
        let mut i: i32 = self.instr.len() as i32 - 1;
        while i >= 0 {
            let current = self.instr[i as usize];

            // skip `Nop` if not following `IfP`/`IfN`
            if current != OpCode::Nop ||
                (current == OpCode::Nop && i > 0 && [OpCode::IfN, OpCode::IfP].contains(&self.instr[(i-1) as usize])) {
                opt_instr.push(current);
            }
            i -= 1;
            if i < 0 { break; }

            // a sequence of instructions modifying `reg_i` which ends in an unconditional `SetI`
            // (i.e. not following `IfP`/`IfN`) can be reduced to the final `SetI`
            let mut was_unconditional_seti = false;
            match self.instr[(i+1) as usize] {
                OpCode::SetI(_) => {
                    match self.instr[i as usize] {
                        OpCode::SetI(_) |
                            OpCode::IncI |
                            OpCode::DecI |
                            OpCode::VtoI |
                            OpCode::Nop => was_unconditional_seti = true,
                        _ => ()
                    };
                },
                _ => ()
            }
            if was_unconditional_seti {
                while i >= 0 {
                    match self.instr[i as usize] {
                        OpCode::SetI(_) |
                            OpCode::IfP |
                            OpCode::IfN |
                            OpCode::DecI |
                            OpCode::IncI |
                            OpCode::VtoI |
                            OpCode::Nop => i -= 1,
                        _ => break
                    }
                }
            }
        }

        opt_instr.reverse();

        let mut jump_table = Program::create_jump_table(&opt_instr);
        if !self.allow_crossing_blocks {
            Program::deactivate_crossing_blocks(&opt_instr, &mut jump_table);
        }

        Program{
            instr: opt_instr,
            num_data_slots: self.num_data_slots,
            jump_table,
            allow_crossing_blocks: self.allow_crossing_blocks
        }
    }
}

pub struct VirtualMachine<'a> {
    /// Virtual machine state.
    state: VmState,
    /// Executed program.
    program: &'a Program,
    /// Handles `Input` and `Output` instructions and evaluates the VM run's end condition.
    io_handler: Option<&'a mut InputOutputHandler>,
}

impl<'a> VirtualMachine<'a> {
    /// Value of `reg_v` after "less than" comparison.
    pub const CMP_LESS: RegValue = -1.0;
    /// Value of `reg_v` after "equal to" comparison.
    pub const CMP_EQUAL: RegValue = 0.0;
    /// Value of `reg_v` after "greater than" comparison.
    pub const CMP_GREATER: RegValue = 1.0;

    ///
    /// Creates a virtual machine instance.
    ///
    /// # Parameters
    ///
    /// * `program` - Program to execute.
    /// * `num_data_slots` - Number of data slots.
    /// * `input_handler` - Called for every `Input` instruction. Receives input number, returns input value.
    /// * `output_handler` - Called for every `Output` instruction. Receives output number and output value.
    ///
    pub fn new(
        program: &'a Program,
        io_handler: Option<&'a mut InputOutputHandler>
    ) -> VirtualMachine<'a> {
        VirtualMachine{
            program,
            io_handler,
            state: VmState{ data: vec![0.0; program.get_num_data_slots()], reg_i: 0, reg_v: 0.0, iptr: 0 }
        }
    }

    pub fn get_state(&self) -> &VmState {
        &self.state
    }

    pub fn set_reg_i(&mut self, reg_i: i32) {
        self.state.reg_i = reg_i;
    }

    pub fn set_reg_v(&mut self, reg_v: RegValue) {
        self.state.reg_v = reg_v;
    }

    pub fn get_data_mut(&mut self) -> &mut [RegValue] {
        &mut self.state.data
    }

    ///
    /// Resets the virtual machine.
    ///
    pub fn reset(&mut self) {
        self.state.reset();
    }

    ///
    /// Runs the program.
    ///
    /// # Parameters
    ///
    /// * `num_exec_instructions` - Max. number of instructions to execute.
    /// * `looped` - If true, program restarts from the beginning after reaching the last instruction.
    /// * `check_end_condition` - If true, `io_handler.check_end_condition()` is called
    /// after every `Output` instruction; if returns true, program execution ends.
    ///
    pub fn run(
        &mut self,
        num_exec_instructions: Option<usize>,
        looped: bool,
        check_end_condition: bool
    ) -> EndReason {
        let mut icounter = 0;
        let instr = self.program.get_instr();
        while num_exec_instructions.is_none() || icounter < num_exec_instructions.unwrap() {
            let opcode = instr[self.state.iptr];
            if self.handle_instruction(opcode) {
                self.state.iptr += 1;
            }
            icounter += 1;
            if self.state.iptr >= instr.len() {
                if looped {
                    self.state.iptr = 0;
                } else {
                    return EndReason::LastInstructionReached;
                }
            }
            if check_end_condition {
                match opcode {
                    OpCode::Output(_) => if self.io_handler.iter().next().unwrap().check_end_condition(icounter) { return EndReason::EndConditionMet; },
                    _ => ()
                }
            }
        }

        EndReason::NumExecInstructions
    }

    ///
    /// Checks if `reg_i` is a valid index into `data`.
    ///
    fn is_data_index(&self) -> bool {
        self.state.reg_i >= 0 && (self.state.reg_i as usize) < self.state.data.len()
    }

    ///
    /// Returns the value of data slot pointed to by `reg_i`.
    ///
    fn data_val(&self) -> RegValue {
        self.state.data[self.state.reg_i as usize]
    }

    ///
    /// Returns `true` if instruction pointer is to be incremented.
    ///
    fn handle_instruction(&mut self, opcode: OpCode) -> bool {
        let jump_table = self.program.get_jump_table();
        match opcode {
            OpCode::SetI(i) => self.state.reg_i = i,

            OpCode::Input(i) => if self.io_handler.is_some() {
                    self.state.reg_v = self.io_handler.iter_mut().next().unwrap().input(i);
                },

            OpCode::Output(i) => if self.io_handler.is_some() {
                    self.io_handler.iter_mut().next().unwrap().output(i, self.state.reg_v);
                },

            OpCode::ItoV => self.state.reg_v = self.state.reg_i as RegValue,

            OpCode::VtoI => self.state.reg_i = self.state.reg_v as i32,

            OpCode::IncV => self.state.reg_v += 1.0,

            OpCode::DecV => self.state.reg_v -= 1.0,

            OpCode::IncI => self.state.reg_i = self.state.reg_i.wrapping_add(1),

            OpCode::DecI => self.state.reg_i = self.state.reg_i.wrapping_sub(1),

            OpCode::Load =>
                if self.is_data_index() {
                    self.state.reg_v = self.state.data[self.state.reg_i as usize];
                },

            OpCode::Store =>
                if self.is_data_index() {
                    self.state.data[self.state.reg_i as usize] = self.state.reg_v;
                },

            OpCode::Swap =>
                if self.is_data_index() {
                    std::mem::swap(&mut self.state.data[self.state.reg_i as usize], &mut self.state.reg_v);
                },

            OpCode::EndGoTo => (),

            OpCode::GoToIfP =>
                if self.state.reg_v >= 0.0 && jump_table[self.state.iptr].is_some() {
                    self.state.iptr = jump_table[self.state.iptr].unwrap();
                    return false;
                },

            OpCode::JumpIfN =>
                if self.state.reg_v < 0.0 && jump_table[self.state.iptr].is_some() {
                    self.state.iptr = jump_table[self.state.iptr].unwrap();
                    return false;
                },

            OpCode::EndJump => (),

            OpCode::IfP => if self.state.reg_v < 0.0 { self.state.iptr += 1; },

            OpCode::IfN => if self.state.reg_v >= 0.0 { self.state.iptr += 1; },

            OpCode::Cmp => if self.is_data_index() {
                let dval = self.data_val();
                if self.state.reg_v < dval { self.state.reg_v = -1.0; }
                else if self.state.reg_v ==  dval { self.state.reg_v = 0.0; }
                else if self.state.reg_v > dval { self.state.reg_v = 1.0; }
            },

            OpCode::Add => if self.is_data_index() { self.state.reg_v += self.data_val(); },

            OpCode::Sub => if self.is_data_index() { self.state.reg_v -= self.data_val(); },

            OpCode::Mul => if self.is_data_index() { self.state.reg_v *= self.data_val(); },

            OpCode::Div => if self.is_data_index() && self.data_val() != 0.0 { self.state.reg_v /= self.data_val(); },

            OpCode::Abs => self.state.reg_v = self.state.reg_v.abs(),

            OpCode::Neg => self.state.reg_v = -self.state.reg_v,

            OpCode::Sqrt => self.state.reg_v = if self.state.reg_v >= 0.0 { self.state.reg_v.sqrt() } else { 0.0 },

            OpCode::Nop => ()
        }

        true
    }
}

macro_rules! t_assert_eq {
    ($expected:expr, $actual:expr) => {
        if $expected != $actual {
            panic!("expected: {}, but was: {}", $expected, $actual);
        }
    };
}

#[cfg(test)]
mod jump_table_tests {
    use super::{OpCode, Program};

    #[test]
    fn simple_goto() {
        let program = Program::new(&[
            OpCode::EndGoTo, // 0: destination of 1
            OpCode::GoToIfP, // 1: should jump to 0
        ], 0, false);

        assert!(
            vec![
                Some(1usize),
                Some(0usize)
            ] == program.get_jump_table());
    }

    #[test]
    fn simple_jump() {
        let program = Program::new(&[
            OpCode::JumpIfN, // 0: should jump to 1
            OpCode::EndJump  // 1: destination of 0
        ], 0, false);

        assert!(
            vec![
                Some(1),
                Some(0),
            ] == program.get_jump_table());
    }

    #[test]
    fn jump_unmatched() {
        let program = Program::new(&[
            OpCode::JumpIfN, // must not jump
            OpCode::Nop
        ], 0, false);

        assert!(
            vec![
                None,
                None
            ] == program.get_jump_table());
    }

    #[test]
    fn goto_unmatched() {
        let program = Program::new(&[
            OpCode::Nop,
            OpCode::GoToIfP, // must not jump
        ], 0, false);

        assert!(
            vec![
                None,
                None
            ] == program.get_jump_table());
    }

    #[test]
    fn goto_unmatched_2() {
        let program = Program::new(&[
            OpCode::EndGoTo, // 0: destination of 1
            OpCode::GoToIfP, // 1: should jump to 0
            OpCode::GoToIfP, // must not jump
        ], 0, false);

        assert!(
            vec![
                Some(1),
                Some(0),
                None
            ] == program.get_jump_table());
    }

    #[test]
    fn jump_unmatched_2() {
        let program = Program::new(&[
            OpCode::JumpIfN, // 0: must not jump
            OpCode::JumpIfN, // 1: should jump to 2
            OpCode::EndJump  // 2: destination of 1
        ], 0, false);

        assert!(
            vec![
                None,
                Some(2),
                Some(1)
            ] == program.get_jump_table());
    }

    #[test]
    fn jump_nested() {
        let program = Program::new(&[
            OpCode::JumpIfN, // 0: should jump to 3
            OpCode::JumpIfN, // 1: should jump to 2
            OpCode::EndJump, // 2: destination of 1
            OpCode::EndJump, // 3: destination of 0
        ], 0, false);

        assert!(
            vec![
                Some(3),
                Some(2),
                Some(1),
                Some(0)
            ] == program.get_jump_table());
    }

    #[test]
    fn goto_nested() {
        let program = Program::new(&[
            OpCode::EndGoTo, // 0: destination of 3
            OpCode::EndGoTo, // 1: destination of 2
            OpCode::GoToIfP, // 2: should jump to 1
            OpCode::GoToIfP, // 3: should jump to 0
        ], 0, false);

        assert!(
            vec![
                Some(3),
                Some(2),
                Some(1),
                Some(0)
            ] == program.get_jump_table());
    }

    #[test]
    fn jump_goto_mixed_1() {
        let program = Program::new(&[
            OpCode::EndGoTo, // 0: destination of 2
            OpCode::JumpIfN, // 1: should jump to 3
            OpCode::GoToIfP, // 2: should jump to 0
            OpCode::EndJump  // 3: destination of 1
        ], 0, true);

        assert!(
            vec![
                Some(2),
                Some(3),
                Some(0),
                Some(1)
            ] == program.get_jump_table());
    }

    #[test]
    fn jump_goto_mixed_2() {
        let program = Program::new(&[
            OpCode::JumpIfN, // 0: should jump to 2
            OpCode::EndGoTo, // 1: destination of 3
            OpCode::EndJump, // 2: destination of 0
            OpCode::GoToIfP  // 3: should jump to 1
        ], 0, true);

        assert!(
            vec![
                Some(2),
                Some(3),
                Some(0),
                Some(1)
            ] == program.get_jump_table());
    }

    #[test]
    fn deact_xing_blks_unchanged() {
        // no crossing blocks, all jumps should remain active
        let program = Program::new(&[
            OpCode::EndGoTo, // 0: destination of 5
            OpCode::EndGoTo, // 1: destination of 2
            OpCode::GoToIfP, // 2: jumps to 1
            OpCode::JumpIfN, // 3: jumps to 4
            OpCode::EndJump, // 4: destination of 3
            OpCode::GoToIfP, // 5: jumps to 0
        ], 0, false);

        assert!(
            vec![
                Some(5),
                Some(2),
                Some(1),
                Some(4),
                Some(3),
                Some(0),
            ] == program.get_jump_table());
    }

    #[test]
    fn deact_xing_blks_jump() {
        let program = Program::new(&[
            OpCode::EndGoTo, // 0: destination of 2
            OpCode::JumpIfN, // 1: crosses 0/2; must not jump
            OpCode::GoToIfP, // 2: jumps to 0
            OpCode::EndJump, // 3: inactive jump target
        ], 0, false);

        assert!(
            vec![
                Some(2),
                None,
                Some(0),
                None
            ] == program.get_jump_table());
    }

    #[test]
    fn deact_xing_blks_goto() {
        let program = Program::new(&[
            OpCode::JumpIfN, // 0: jumps to 2
            OpCode::EndGoTo, // 1: inactive jump target
            OpCode::EndJump, // 2: destination of 0
            OpCode::GoToIfP, // 3: crosses 0/2; must not jump
        ], 0, false);

        assert!(
            vec![
                Some(2),
                None,
                Some(0),
                None
            ] == program.get_jump_table());
    }

    #[test]
    fn deact_xing_blks_goto_multiple_1() {
        let program = Program::new(&[
            OpCode::JumpIfN, // 0: jumps to 4
            OpCode::JumpIfN, // 1: jumps to 3
            OpCode::EndGoTo, // 2: inactive jump target
            OpCode::EndJump, // 3: destination of 1
            OpCode::EndJump, // 4: destination of 0
            OpCode::GoToIfP, // 5: crosses 0/4 and 1/3; must not jump
        ], 0, false);

        assert!(
            vec![
                Some(4),
                Some(3),
                None,
                Some(1),
                Some(0),
                None
            ] == program.get_jump_table());
    }

    #[test]
    fn deact_xing_blks_goto_multiple_2() {
        let program = Program::new(&[
            OpCode::JumpIfN, // 0: jumps to 3
            OpCode::EndGoTo, // 1: inactive jump target
            OpCode::EndGoTo, // 2: inactive jump target
            OpCode::EndJump, // 3: destination of 0
            OpCode::GoToIfP, // 4: crosses 0/3; must not jump
            OpCode::GoToIfP, // 5: crosses 0/3; must not jump
        ], 0, false);

        assert!(
            vec![
                Some(3),
                None,
                None,
                Some(0),
                None,
                None
            ] == program.get_jump_table());
    }


    #[test]
    fn deact_xing_blks_jump_multiple_1() {
        let program = Program::new(&[
            OpCode::EndGoTo, // 0: destination of 4
            OpCode::EndGoTo, // 1: destination of 3
            OpCode::JumpIfN, // 2: crosses 0/4 and 1/3; must not jump
            OpCode::GoToIfP, // 3: jumps to 1
            OpCode::GoToIfP, // 4: jumps to 0
            OpCode::EndJump, // 5: inactive jump target
        ], 0, false);

        assert!(
            vec![
                Some(4),
                Some(3),
                None,
                Some(1),
                Some(0),
                None
            ] == program.get_jump_table());
    }

    #[test]
    fn deact_xing_blks_jump_multiple_2() {
        let program = Program::new(&[
            OpCode::EndGoTo, // 0: destination of 3
            OpCode::JumpIfN, // 1: crosses 0/3; must not jump
            OpCode::JumpIfN, // 2: crosses 0/3; must not jump
            OpCode::GoToIfP, // 3: jumps to 0
            OpCode::EndJump, // 4: inactive jump target
            OpCode::EndJump, // 5: inactive jump target
        ], 0, false);

        assert!(
            vec![
                Some(3),
                None,
                None,
                Some(0),
                None,
                None
            ] == program.get_jump_table());
    }
}

#[cfg(test)]
mod instruction_tests {
    use super::{InputOutputHandler, OpCode, Program, RegValue, VirtualMachine};

    #[test]
    fn set_i() {
        const INT_VAL: i32 = 55;
        let program = Program::new(&[OpCode::SetI(INT_VAL)], 1, false);
        let mut vm = VirtualMachine::new(&program, None);

        t_assert_eq!(0, vm.get_state().reg_i);
        vm.run(None, false, false);
        t_assert_eq!(INT_VAL, vm.get_state().reg_i);
        t_assert_eq!(1, vm.get_state().iptr);
    }

    struct InputHandler {
        expected_input_num: i32,
        input_val: RegValue
    }

    impl InputOutputHandler for InputHandler {
        fn input(&mut self, input_num: i32) -> RegValue {
            t_assert_eq!(self.expected_input_num, input_num);
            self.input_val
        }

        fn output(&mut self, _output_num: i32, _output_val: RegValue) { }

        fn check_end_condition(&self, _num_execd_instructions: usize) -> bool { false }
    }

    #[test]
    fn input() {
        const INPUT_NUM: i32 = 55;
        const INPUT_VAL: RegValue = 7.0;
        let mut ih = InputHandler{ expected_input_num: INPUT_NUM, input_val: INPUT_VAL };
        let program = Program::new(&[OpCode::Input(INPUT_NUM)], 1, false);
        let mut vm = VirtualMachine::new(&program, Some(&mut ih));

        vm.run(None, false, false);
        t_assert_eq!(INPUT_VAL, vm.get_state().reg_v);
    }

    struct OutputHandler {
        pub called: bool
    }

    impl InputOutputHandler for OutputHandler {
        fn input(&mut self, _input_num: i32) -> RegValue { 0.0 }

        fn output(&mut self, _output_num: i32, _output_val: RegValue) {
            self.called = true;
        }

        fn check_end_condition(&self, _num_execd_instructions: usize) -> bool { false }
    }

    #[test]
    fn output_i_to_v() {
        const OUTPUT_NUM: i32 = 55;
        const OUTPUT_VAL: RegValue = 7.0;
        let program = Program::new(&[
            OpCode::SetI(OUTPUT_VAL as i32),
            OpCode::ItoV,
            OpCode::Output(OUTPUT_NUM)
        ], 1, false);
        let mut oh = OutputHandler{ called: false };
        {
            let mut vm = VirtualMachine::new(&program, Some(&mut oh));
            vm.run(None, false, false);
        }
        assert!(oh.called);
    }

    #[test]
    fn v_to_i() {
        const EXPECTED_VAL: RegValue = 55.5;
        let program = Program::new(&[OpCode::VtoI], 0, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_i(0);
        vm.set_reg_v(EXPECTED_VAL);

        vm.run(None, false, false);
        t_assert_eq!(EXPECTED_VAL as i32, vm.get_state().reg_i);
    }

    #[test]
    fn inc_v() {
        const INITIAL_VAL: RegValue = 5.0;
        let program = Program::new(&[OpCode::IncV], 0, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(INITIAL_VAL);

        vm.run(None, false, false);
        t_assert_eq!(INITIAL_VAL + 1.0 as RegValue, vm.get_state().reg_v);
    }

    #[test]
    fn dec_v() {
        const INITIAL_VAL: RegValue = 5.0;
        let program = Program::new(&[OpCode::DecV], 0, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(INITIAL_VAL);

        vm.run(None, false, false);
        t_assert_eq!(INITIAL_VAL - 1.0 as RegValue, vm.get_state().reg_v);
    }

    #[test]
    fn inc_i() {
        const INITIAL_VAL: i32 = 5;
        let program = Program::new(&[OpCode::IncI], 0, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_i(INITIAL_VAL);

        vm.run(None, false, false);
        t_assert_eq!(INITIAL_VAL + 1, vm.get_state().reg_i);
    }

    #[test]
    fn dec_i() {
        const INITIAL_VAL: i32 = 5;
        let program = Program::new(&[OpCode::DecI], 0, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_i(INITIAL_VAL);

        vm.run(None, false, false);
        t_assert_eq!(INITIAL_VAL - 1, vm.get_state().reg_i);
    }

    #[test]
    fn load() {
        const INITIAL_VAL: RegValue = 5.0;
        const REG_NUM: usize = 0;
        let program = Program::new(&[
            OpCode::SetI(REG_NUM as i32),
            OpCode::Load
        ], REG_NUM + 1, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.get_data_mut()[REG_NUM] = INITIAL_VAL;

        vm.run(None, false, false);
        t_assert_eq!(INITIAL_VAL, vm.get_state().reg_v);
    }

    #[test]
    fn store() {
        const STORE_VAL: RegValue = 5.0;
        const REG_NUM: usize = 0;
        let program = Program::new(&[
            OpCode::SetI(REG_NUM as i32),
            OpCode::Store
        ], REG_NUM + 1, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(STORE_VAL);

        vm.run(None, false, false);
        t_assert_eq!(STORE_VAL, vm.get_state().data[REG_NUM]);
    }

    #[test]
    fn swap() {
        const DATA_VAL: RegValue = 11.0;
        const REG_VAL: RegValue = 55.0;
        const REG_NUM: usize = 0;
        let program = Program::new(&[
            OpCode::SetI(REG_NUM as i32),
            OpCode::Swap
        ], REG_NUM + 1, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(REG_VAL);
        vm.get_data_mut()[REG_NUM] = DATA_VAL;

        vm.run(None, false, false);
        t_assert_eq!(REG_VAL, vm.get_state().data[REG_NUM]);
        t_assert_eq!(DATA_VAL, vm.get_state().reg_v);
    }

    #[test]
    fn goto_if_p() {
        let program = Program::new(&[
            OpCode::EndGoTo,
            OpCode::SetI(1),
            OpCode::ItoV,
            OpCode::GoToIfP // jumps back to the first instruction
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(Some(4), false, false);
        t_assert_eq!(0, vm.get_state().iptr);
    }

    #[test]
    fn jump_if_n() {
        const EXPECTED_VAL: i32 = -99;
        let program = Program::new(&[
            OpCode::SetI(EXPECTED_VAL),
            OpCode::ItoV,
            OpCode::JumpIfN,
            OpCode::SetI(10),
            OpCode::EndJump
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(None, false, false);
        t_assert_eq!(EXPECTED_VAL, vm.get_state().reg_i);
    }

    #[test]
    fn if_p_true() {
        const EXPECTED_VAL: i32 = 10;
        let program = Program::new(&[
            OpCode::SetI(1),
            OpCode::ItoV,
            OpCode::IfP,
            OpCode::SetI(EXPECTED_VAL),
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(None, false, false);
        t_assert_eq!(EXPECTED_VAL, vm.get_state().reg_i);
    }

    #[test]
    fn if_p_false() {
        const EXPECTED_VAL: i32 = -10;
        let program = Program::new(&[
            OpCode::SetI(EXPECTED_VAL),
            OpCode::ItoV,
            OpCode::IfP,
            OpCode::SetI(1),
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(None, false, false);
        t_assert_eq!(EXPECTED_VAL, vm.get_state().reg_i);
    }

    #[test]
    fn if_n_true() {
        const EXPECTED_VAL: i32 = 10;
        let program = Program::new(&[
            OpCode::SetI(-1),
            OpCode::ItoV,
            OpCode::IfN,
            OpCode::SetI(EXPECTED_VAL),
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(None, false, false);
        t_assert_eq!(EXPECTED_VAL, vm.get_state().reg_i);
    }

    #[test]
    fn if_n_false() {
        const EXPECTED_VAL: i32 = 10;
        let program = Program::new(&[
            OpCode::SetI(EXPECTED_VAL),
            OpCode::ItoV,
            OpCode::IfN,
            OpCode::SetI(1),
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(None, false, false);
        t_assert_eq!(EXPECTED_VAL, vm.get_state().reg_i);
    }

    #[test]
    fn cmp_less() {
        let program = Program::new(&[
            OpCode::SetI(1),
            OpCode::ItoV,
            OpCode::SetI(0),
            OpCode::Store,  // now data[0] == 1
            OpCode::SetI(0),
            OpCode::ItoV,  // now reg_v == 0
            OpCode::Cmp
        ], 1, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(None, false, false);
        t_assert_eq!(VirtualMachine::CMP_LESS, vm.get_state().reg_v);
    }

    #[test]
    fn cmp_equal() {
        let program = Program::new(&[
            OpCode::SetI(1),
            OpCode::ItoV,
            OpCode::SetI(0),
            OpCode::Store,  // now data[0] == 1
            OpCode::SetI(1),
            OpCode::ItoV,  // now reg_v == 1.0
            OpCode::SetI(0),
            OpCode::Cmp
        ], 1, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(None, false, false);
        t_assert_eq!(VirtualMachine::CMP_EQUAL, vm.get_state().reg_v);
    }

    #[test]
    fn cmp_greater() {
        let program = Program::new(&[
            OpCode::SetI(1),
            OpCode::ItoV,
            OpCode::SetI(0),
            OpCode::Store,  // now data[0] == 1
            OpCode::SetI(2),
            OpCode::ItoV,  // now reg_v == 2.0
            OpCode::SetI(0),
            OpCode::Cmp
        ], 1, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(None, false, false);
        t_assert_eq!(VirtualMachine::CMP_GREATER, vm.get_state().reg_v);
    }

    #[test]
    fn cmp_data_idx_out_of_range() {
        const INITIAL_VALUE: RegValue = 55.0;
        let program = Program::new(&[
            OpCode::SetI(INITIAL_VALUE as i32),
            OpCode::ItoV,
            OpCode::Cmp  // no change, data[INITIAL_VALUE] does not exist
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.run(None, false, false);
        t_assert_eq!(INITIAL_VALUE, vm.get_state().reg_v);
    }

    #[test]
    fn add() {
        let program = Program::new(&[
            OpCode::Add
        ], 1, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(11.0);
        vm.get_data_mut()[0] = 22.0;

        vm.run(None, false, false);
        t_assert_eq!(11.0 + 22.0, vm.get_state().reg_v);
    }

    #[test]
    fn sub() {
        let program = Program::new(&[
            OpCode::Sub
        ], 1, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(11.0);
        vm.get_data_mut()[0] = 22.0;


        vm.run(None, false, false);
        t_assert_eq!(11.0 - 22.0, vm.get_state().reg_v);
    }

    #[test]
    fn mul() {
        let program = Program::new(&[
            OpCode::Mul
        ], 1, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(11.0);
        vm.get_data_mut()[0] = 22.0;

        vm.run(None, false, false);
        t_assert_eq!(11.0 * 22.0, vm.get_state().reg_v);
    }

    #[test]
    fn div() {
        let program = Program::new(&[
            OpCode::Div
        ], 1, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(11.0);
        vm.get_data_mut()[0] = 22.0;

        vm.run(None, false, false);
        t_assert_eq!(11.0 / 22.0, vm.get_state().reg_v);
    }

    #[test]
    fn div_by_zero() {
        let program = Program::new(&[
            OpCode::Div
        ], 1, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(11.0);
        vm.get_data_mut()[0] = 0.0;

        vm.run(None, false, false);
        t_assert_eq!(11.0, vm.get_state().reg_v);  // division by zero has no effect
    }

    #[test]
    fn abs() {
        let program = Program::new(&[
            OpCode::Abs
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.set_reg_v(11.0);
        vm.run(None, false, false);
        t_assert_eq!(11.0, vm.get_state().reg_v);

        vm.reset();

        vm.set_reg_v(-11.0);
        vm.run(None, false, false);
        t_assert_eq!(11.0, vm.get_state().reg_v);
    }

    #[test]
    fn neg() {
        let program = Program::new(&[
            OpCode::Neg
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.set_reg_v(11.0);

        vm.run(None, false, false);
        t_assert_eq!(-11.0, vm.get_state().reg_v);
    }

    #[test]
    fn sqrt() {
        let program = Program::new(&[
            OpCode::Sqrt
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.set_reg_v(11.0);
        vm.run(None, false, false);
        t_assert_eq!(11.0f32.sqrt(), vm.get_state().reg_v);
    }

    #[test]
    fn sqrt_negative() {
        let program = Program::new(&[
            OpCode::Sqrt
        ], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        vm.set_reg_v(-11.0);
        vm.run(None, false, false);
        t_assert_eq!(0.0, vm.get_state().reg_v);
    }

    #[test]
    fn nop() {
        let program = Program::new(&[
            OpCode::Nop
        ], 4, false);
        let mut vm = VirtualMachine::new(&program, None);
        vm.get_data_mut()[0] = 0.0;
        vm.get_data_mut()[1] = 1.0;
        vm.get_data_mut()[2] = 2.0;
        vm.get_data_mut()[3] = 3.0;

        let state_pre = vm.get_state().clone();
        vm.run(None, false, false);
        let state_post = vm.get_state();

        for i in 0..state_pre.data.len() {
            t_assert_eq!(state_pre.data[i], state_post.data[i]);
        }
        t_assert_eq!(state_pre.reg_i, state_post.reg_i);
        t_assert_eq!(state_pre.reg_v, state_post.reg_v);
        t_assert_eq!(state_pre.iptr + 1, state_post.iptr);
    }
}

#[cfg(test)]
mod end_condition_tests {
    use super::{EndReason, InputOutputHandler, OpCode, Program, RegValue, VirtualMachine};

    #[test]
    fn last_instr_reached() {
        let program = Program::new(&[OpCode::Nop], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        let reason = vm.run(None, false, false);
        t_assert_eq!(EndReason::LastInstructionReached, reason);
    }

    #[test]
    fn num_exec_instructions() {
        let program = Program::new(&[OpCode::Nop], 0, false);
        let mut vm = VirtualMachine::new(&program, None);

        let reason = vm.run(Some(100), true, false);
        t_assert_eq!(EndReason::NumExecInstructions, reason);
    }

    #[test]
    fn end_condition_met() {
        const NUM_INSTR_TO_RUN: usize = 100;
        const NUM_INSTR_TO_END: usize = 50;

        #[derive(Default)]
        struct IoHandler { }
        impl InputOutputHandler for IoHandler {
            fn input(&mut self, _: i32) -> RegValue { 0.0 }
            fn output(&mut self, _: i32, _: RegValue) { }
            fn check_end_condition(&self, num_execd_instructions: usize) -> bool {
                num_execd_instructions > NUM_INSTR_TO_END
            }
        }

        let mut io_handler = IoHandler::default();

        let program = Program::new(&[OpCode::Output(0)], 0, false);
        let mut vm = VirtualMachine::new(&program, Some(&mut io_handler));

        let reason = vm.run(Some(NUM_INSTR_TO_RUN), true, true);
        t_assert_eq!(EndReason::EndConditionMet, reason);
    }

    #[test]
    fn end_condition_not_met() {
        const NUM_INSTR_TO_RUN: usize = 100;
        const NUM_INSTR_TO_END: usize = 200;

        #[derive(Default)]
        struct IoHandler { }
        impl InputOutputHandler for IoHandler {
            fn input(&mut self, _: i32) -> RegValue { 0.0 }
            fn output(&mut self, _: i32, _: RegValue) { }
            fn check_end_condition(&self, num_execd_instructions: usize) -> bool {
                num_execd_instructions > NUM_INSTR_TO_END
            }
        }

        let mut io_handler = IoHandler::default();

        let program = Program::new(&[OpCode::Output(0)], 0, false);
        let mut vm = VirtualMachine::new(&program, Some(&mut io_handler));

        let reason = vm.run(Some(NUM_INSTR_TO_RUN), true, true);
        t_assert_eq!(EndReason::NumExecInstructions, reason);
    }
}

#[cfg(test)]
mod optimization_tests {
    use vm::{OpCode, Program};

    #[test]
    fn seti() {
        let prog = Program::new(
            &[
                OpCode::SetI(0), // should be optimized out
                OpCode::SetI(1), //
                OpCode::SetI(2), //
                OpCode::SetI(3)
            ],
            1, false);
        let opt_prog = prog.get_optimized();

        assert!(opt_prog.get_instr() == &[OpCode::SetI(3)]);
        t_assert_eq!(prog.get_num_data_slots(), opt_prog.get_num_data_slots());
    }

    #[test]
    fn seti_short() {
        let prog = Program::new(
            &[
                OpCode::SetI(0),
            ],
            1, false);
        let opt_prog = prog.get_optimized();

        assert!(opt_prog.get_instr() == &[OpCode::SetI(0)]);
    }

    #[test]
    fn seti_conditional_1() {
        let prog = Program::new(
            &[
                OpCode::Add,
                OpCode::IfP,         // should be optimized out
                    OpCode::SetI(1), //
                OpCode::SetI(2),     //
                OpCode::IfN,         //
                    OpCode::SetI(3), //
                OpCode::SetI(4),
                OpCode::Add,
            ],
            1, false);
        let opt_prog = prog.get_optimized();

        assert!(opt_prog.get_instr() == &[
            OpCode::Add,
            OpCode::SetI(4),
            OpCode::Add,
        ]);
    }

    #[test]
    fn seti_conditional_2() {
        let prog = Program::new(
            &[
                OpCode::Add,
                OpCode::IfP,         // should be optimized out
                    OpCode::SetI(1), //
                OpCode::SetI(2),
                OpCode::Add,
                OpCode::Nop,         // should be optimized out
                OpCode::IfN,         //
                    OpCode::SetI(3), //
                OpCode::SetI(4),
                OpCode::Add,
            ],
            1, false);
        let opt_prog = prog.get_optimized();

        assert!(opt_prog.get_instr() == &[
            OpCode::Add,
            OpCode::SetI(2),
            OpCode::Add,
            OpCode::SetI(4),
            OpCode::Add,
        ]);
    }

    #[test]
    fn seti_conditional_3() {
        let prog = Program::new(
            &[
                OpCode::SetI(0),  // should be optimized out
                OpCode::SetI(1),
                OpCode::IfP,
                    OpCode::SetI(2),
            ],
            1, false);
        let opt_prog = prog.get_optimized();

        assert!(opt_prog.get_instr() == &[
            OpCode::SetI(1),
            OpCode::IfP,
                OpCode::SetI(2),
        ]);
    }

    #[test]
    fn modify_reg_i_no_optimizations_1() {
        let prog = Program::new(
            &[
                OpCode::SetI(0),
                OpCode::Add
            ],
            1, false);
        let opt_prog = prog.get_optimized();

        assert!(opt_prog.get_instr() == &[
            OpCode::SetI(0),
            OpCode::Add
        ]);
    }

    #[test]
    fn modify_reg_i_no_optimizations_2() {
        let prog = Program::new(
            &[
                OpCode::IfP,
                    OpCode::SetI(0)
            ],
            1, false);
        let opt_prog = prog.get_optimized();

        assert!(opt_prog.get_instr() == &[
            OpCode::IfP,
                OpCode::SetI(0)
        ]);
    }

    #[test]
    fn modify_reg_i() {
        let prog = Program::new(
            &[
                OpCode::DecI,  // should be optimized out
                OpCode::VtoI,  //
                OpCode::Nop,   //
                OpCode::IncI,  //
                OpCode::SetI(0),
            ],
            1, false);
        let opt_prog = prog.get_optimized();

        assert!(opt_prog.get_instr() == &[
            OpCode::SetI(0)
        ]);
    }

    #[test]
    fn remove_nop() {
        let prog = Program::new(
            &[
                OpCode::Nop,  // should be optimized out
                OpCode::Nop,  //
                OpCode::Add,
                OpCode::IfP,
                    OpCode::Nop,
                OpCode::Nop,  //
                OpCode::IfN,
                    OpCode::Nop
            ],
            1, false);
        let opt_prog = prog.get_optimized();

        assert!(opt_prog.get_instr() == &[
            OpCode::Add,
            OpCode::IfP,
                OpCode::Nop,
            OpCode::IfN,
                OpCode::Nop
        ]);
    }
}