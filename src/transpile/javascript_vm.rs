//
// genetic - genetic programming experiments
// Copyright (c) 2019 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//
//
// File description:
//   Module: transpiling to JavaScript virtual machine with an embedded program.
//

use vm;

/// Creates a virtual machine in JavaScript with `program` embedded in it.
pub fn program_to_javascript_vm(program: &vm::Program) -> String {
    FIRST_PART.to_string() +
        &generate_instruction_list(program) +
        &generate_jump_table(program) +
        &generate_data_slots(program) +
        &SECOND_PART.to_string()
}

/// Number of jump table and instruction items per line in the output JS code.
const ITEMS_PER_LINE: usize = 8;

/// Generates the data slots array's definition.
fn generate_data_slots(program: &vm::Program) -> String {
    let mut result = "        this.data = [\n            ".to_string();
    for i in 1..=program.get_num_data_slots() {
        result += &"0.0, ";
        if i % ITEMS_PER_LINE == 0 { result += &"\n            "; }
    }
    result += &"\n        ];\n";

    result
}

/// Generates the contents of the VM's `this.jumpTable` array.
fn generate_jump_table(program: &vm::Program) -> String {
    let mut jump_table = "        this.jumpTable = [\n            ".to_string();
    for (i, jitem) in program.get_jump_table().iter().enumerate() {
        match jitem {
            Some(jmp_target) => jump_table += &format!("{}, ", jmp_target),
            None             => jump_table +=         &"null, "
        }
        if (i+1) % ITEMS_PER_LINE == 0 { jump_table += &"\n            "; }
    }
    jump_table += "\n        ];\n";

    jump_table
}

/// Generates the contents of the VM's `this.instructions` array.
fn generate_instruction_list(program: &vm::Program) -> String {
    let mut instructions = "        this.instructions = [\n            ".to_string();
    for (i, instr) in program.get_instr().iter().enumerate() {
        let instr_str =
            match instr {
                vm::OpCode::SetI(i)   => format!("new SetI({}), ", i),
                vm::OpCode::Input(i)  => format!("new Input({}), ", i),
                vm::OpCode::Output(i) => format!("new Output({}), ", i),
                vm::OpCode::ItoV              => "new ItoV, ".to_string(),
                vm::OpCode::VtoI              => "new VtoI, ".to_string(),
                vm::OpCode::IncV              => "new IncV, ".to_string(),
                vm::OpCode::DecV              => "new DecV, ".to_string(),
                vm::OpCode::IncI              => "new IncI, ".to_string(),
                vm::OpCode::DecI              => "new DecI, ".to_string(),
                vm::OpCode::Load              => "new Load, ".to_string(),
                vm::OpCode::Store             => "new Store, ".to_string(),
                vm::OpCode::Swap              => "new Swap, ".to_string(),
                vm::OpCode::EndGoTo           => "new EndGoTo, ".to_string(),
                vm::OpCode::GoToIfP           => "new GoToIfP, ".to_string(),
                vm::OpCode::JumpIfN           => "new JumpIfN, ".to_string(),
                vm::OpCode::EndJump           => "new EndJump, ".to_string(),
                vm::OpCode::IfP               => "new IfP, ".to_string(),
                vm::OpCode::IfN               => "new IfN, ".to_string(),
                vm::OpCode::Cmp               => "new Cmp, ".to_string(),
                vm::OpCode::Add               => "new Add, ".to_string(),
                vm::OpCode::Sub               => "new Sub, ".to_string(),
                vm::OpCode::Mul               => "new Mul, ".to_string(),
                vm::OpCode::Div               => "new Div, ".to_string(),
                vm::OpCode::Abs               => "new Abs, ".to_string(),
                vm::OpCode::Neg               => "new Neg, ".to_string(),
                vm::OpCode::Sqrt              => "new Sqrt, ".to_string(),
                vm::OpCode::Nop               => "new Nop, ".to_string(),
            };
        instructions += &instr_str;
        if (i+1) % ITEMS_PER_LINE == 0 { instructions += &"\n            "; }
    }
    instructions += &"\n        ];\n";

    instructions
}

///
/// First part of the output JavaScript code.
///
/// Strings returned by `generate_instruction_list`, 'generate_data_slots`
/// and `generate_jump_table` are inserted between `FIRST_PART` and `SECOND_PART`.
///
const FIRST_PART: &str =r#"
"use strict";

// virtual machine instruction opcodes
class SetI { constructor(i) { this.i = i; } };
class Input { constructor(i) { this.i = i; } };
class Output { constructor(i) { this.i = i; } };
class ItoV { };
class VtoI { };
class IncV { };
class DecV { };
class IncI { };
class DecI { };
class Load { };
class Store { };
class Swap { };
class EndGoTo { };
class GoToIfP { };
class JumpIfN { };
class EndJump { };
class IfP { };
class IfN { };
class Cmp { };
class Add { };
class Sub { };
class Mul { };
class Div { };
class Abs { };
class Neg { };
class Sqrt { };
class Nop { };

/**
 * @callback VmInputHandler
 * @param {number} inputNumber - Input number (integer).
 * @returns {number} - Input value.
 */

 /**
 * @callback VmOutputHandler
 * @param {number} outputNumber - Output number (integer).
 * @param {number} outputValue
 */

/** Virtual machine running a hard-coded program. */
class VM {
    /**
     * @callback {VmInputHander} inputHandler - Called for `Input` instructions. May be `null`.
     * @callback {VmOutputHander} outputHandler - Called for `Output` instructions. May be `null`.
     */
    constructor(inputHandler, outputHandler) {
"#;

///
/// Second (and the last) part of the output JavaScript code.
///
/// Strings returned by `generate_instruction_list`, 'generate_data_slots`
/// and `generate_jump_table` are inserted between `FIRST_PART` and `SECOND_PART`.
///
const SECOND_PART: &str = r#"

        this.iptr = 0;
        this.regI = 0;
        this.regV = 0.0;

        this.inputHandler = inputHandler;
        this.outputHandler = outputHandler;
    }

    /** Executes the specified number of instructions. Subsequent calls resume execution where it stopped. */
    run(num_instructions) {
        let icounter = 0;
        while (icounter < num_instructions) {
            if (this.handleInstruction(this.instructions[this.iptr])) {
                this.iptr += 1;
            }
            icounter += 1;
            if (this.iptr >= this.instructions.length) {
                this.iptr = 0;
            }
        }
    }

    /** Executes the program until the `end_condition` function returns `true`. Subsequent calls resume execution where it stopped. */
    runUntil(end_condition) {
        while (!end_condition()) {
            if (this.handleInstruction(this.instructions[this.iptr])) {
                this.iptr += 1;
            }
            if (this.iptr >= this.instructions.length) {
                this.iptr = 0;
            }
        }
    }

    isDataIndex() {
        return this.regI >= 0 && this.regI < this.data.length;
    }

    /** Handles `instr`; returns `true` if instruction pointer is to be incremented by the caller afterwards. */
    handleInstruction(instr) {
        if (instr instanceof SetI) { this.regI = instr.i; }
        else if (instr instanceof Input) { if (this.inputHandler != null) this.regV = this.inputHandler(instr.i); }
        else if (instr instanceof Output) { if (this.outputHandler != null) this.outputHandler(instr.i, this.regV); }
        else if (instr instanceof ItoV) { this.regV = this.regI; }
        else if (instr instanceof VtoI) { this.regI = Math.trunc(this.regV); }
        else if (instr instanceof IncV) { this.regV += 1.0 }
        else if (instr instanceof DecV) { this.regV -= 1.0 }
        else if (instr instanceof IncI) { this.regI += 1; if (this.regI >= 0x80000000) this.regI = -1; }
        else if (instr instanceof DecI) { this.regI -= 1; if (this.regI < -0x80000000) this.regI = 0x7FFFFFFF; }
        else if (instr instanceof Load) { if (this.isDataIndex()) this.regV = this.data[this.regI]; }
        else if (instr instanceof Store) { if (this.isDataIndex()) this.data[this.regI] = this.regV; }
        else if (instr instanceof Swap) {
            if (this.isDataIndex()) {
                let tmp = this.regV;
                this.regV = this.data[this.regI];
                this.data[this.regI] = tmp;
            }
        }
        else if (instr instanceof EndGoTo) { }
        else if (instr instanceof GoToIfP) {
            if (this.regV >= 0.0 && this.jumpTable[this.iptr] != null) {
                this.iptr = this.jumpTable[this.iptr];
                return false;
            }
        }
        else if (instr instanceof JumpIfN) {
            if (this.regV < 0.0 && this.jumpTable[this.iptr] != null) {
                this.iptr = this.jumpTable[this.iptr];
                return false;
            }
        }
        else if (instr instanceof EndJump) { }
        else if (instr instanceof IfP) { if (this.regV < 0.0) this.iptr += 1; }
        else if (instr instanceof IfN) { if (this.regV >= 0.0) this.iptr += 1; }
        else if (instr instanceof Cmp) {
            if (this.isDataIndex()) {
                let dval = this.data[this.regI];
                if (this.regV < dval) this.regV = -1.0;
                else if (this.regV == dval) this.regV = 0.0;
                else if (this.regV > dval) this.regV = 1.0;
            }
        }
        else if (instr instanceof Add) { if (this.isDataIndex()) this.regV += this.data[this.regI]; }
        else if (instr instanceof Sub) { if (this.isDataIndex()) this.regV -= this.data[this.regI]; }
        else if (instr instanceof Mul) { if (this.isDataIndex()) this.regV *= this.data[this.regI]; }
        else if (instr instanceof Div) { if (this.isDataIndex() && this.data[this.regI] != 0.0) this.regV /= this.data[this.regI]; }
        else if (instr instanceof Abs) { this.regV = Math.abs(this.regV); }
        else if (instr instanceof Neg) { this.regV = -this.regV; }
        else if (instr instanceof Sqrt) { if (this.regV >= 0.0) this.regV = Math.sqrt(this.regV); else this.regV = 0.0; }
        else if (instr instanceof Nop) { }

        return true;
    }
}
"#;