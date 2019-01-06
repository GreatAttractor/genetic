
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
        this.instructions = [
            new Input(1), new Mul, new SetI(4), new Sub, new Sub, new Input(3), new Output(0), new SetI(3), 
            new Store, new Input(0), new Abs, new Output(2), new Div, new EndJump, new Output(1), new IncI, 
            new Input(2), new Input(2), new SetI(3), new Add, new EndGoTo, new SetI(4), new Cmp, new Sub, 
            new EndGoTo, new Output(2), new DecV, new EndJump, new Output(1), new GoToIfP, new Div, new Sub, 
            new Sub, new Input(3), new Output(0), new EndGoTo, new Store, new SetI(3), new Abs, new Input(0), 
            new Output(2), new Div, new Output(1), new EndJump, new SetI(3), new Input(2), new Add, new EndGoTo, 
            new Input(2), new SetI(4), new Sub, new EndGoTo, new EndJump, new Output(2), new DecV, new Output(0), 
            new Store, new GoToIfP, new Sub, new Store, new SetI(4), new EndGoTo, new JumpIfN, new IncV, 
            new JumpIfN, new Add, new IncI, new Swap, new Neg, new Output(3), new Add, new JumpIfN, 
            new EndJump, new Add, new SetI(3), new Sub, new GoToIfP, new SetI(4), new Input(2), new EndJump, 
            new Store, new Load, new Sub, new SetI(3), new IncI, new Input(0), new Input(0), new IncV, 
            new JumpIfN, new Div, new Output(1), new EndJump, new Input(2), new SetI(0), new IncI, new Output(2), 
            new Input(0), new Output(2), new Input(0), new Output(1), new Div, new IncV, new EndGoTo, new JumpIfN, 
            new Output(1), new Add, new EndJump, new Div, new IfP, new IncI, new Output(2), new Output(2), 
            new Input(0), new Input(0), new Output(2), new Div, new Input(0), new Output(1), new EndGoTo, new IncI, 
            new Output(3), new Neg, new Add, new EndJump, new SetI(3), new EndJump, new SetI(3), new Add, 
            new Input(1), new Sub, new GoToIfP, new Input(2), new SetI(4), new EndJump, new GoToIfP, new Input(3), 
            new Load, new Sub, new Store, new SetI(3), new IncI, new Input(0), new Input(0), new IncV, 
            new JumpIfN, new IncV, new Add, new SetI(4), new EndJump, new Abs, new IncV, new Swap, 
            new SetI(2), new EndGoTo, new ItoV, new JumpIfN, new EndJump, new SetI(2), new Add, new Div, 
            new IfP, new IncI, new Output(1), new DecI, new Input(2), new EndJump, new Input(2), new SetI(2), 
            new IfP, new IncI, new Output(2), new Input(0), new Output(2), new Div, new Input(0), new Output(1), 
            new EndGoTo, new IncV, new JumpIfN, new EndJump, new SetI(2), new Div, new IfP, new Sub, 
            new SetI(0), new Input(0), new IncV, new Store, new Output(2), new Input(0), new Store, new Output(1), 
            new DecV, new Add, new Sub, new Store, new Sub, new Add, new DecV, new IncI, 
            new IncI, new Swap, new Neg, new Output(3), new Add, new Div, new Store, new Output(3), 
            new EndGoTo, new EndJump, new Output(2), new DecV, new Output(0), new Store, new GoToIfP, new EndGoTo, 
            new SetI(4), new EndGoTo, new JumpIfN, new JumpIfN, new Add, new IncI, new Input(0), new Swap, 
            new Neg, new Output(3), new EndJump, new SetI(3), new Input(2), new SetI(2), new IncI, new Input(0), 
            new IfP, new Output(2), new Input(0), new Output(2), new Input(0), new Div, new Output(1), new IncV, 
            new EndGoTo, new JumpIfN, new Add, new EndJump, new SetI(2), new Div, new Output(1), new IncI, 
            new Output(2), new Output(2), new Input(0), new Input(0), new Output(2), new Input(0), new Div, new Output(1), 
            new EndGoTo, new IncI, new Add, new Neg, new Output(3), new Add, new EndJump, new Add, 
            new SetI(3), new Input(1), new SetI(3), new Sub, new GoToIfP, new Input(2), new SetI(4), new EndJump, 
            new GoToIfP, new Input(3), new Store, new Load, new Sub, new IncI, new GoToIfP, new Swap, 
            new Neg, new Output(3), new Add, new Div, new Store, new EndJump, new SetI(3), new Add, 
            new Input(1), new GoToIfP, new Input(2), new Input(0), new Input(0), new Input(0), new Div, new EndGoTo, 
            new IncV, new Output(2), new Input(0), new Input(0), new Div, new Output(1), new Add, new SetI(3), 
            new Add, new Input(1), new Sub, new GoToIfP, new Input(2), new SetI(4), new EndJump, new GoToIfP, 
            new SetI(2), new IfP, new IncI, new EndJump, new Output(2), new Store, new Output(2), new Input(0), 
            new Output(1), new EndGoTo, new Output(1), new JumpIfN, new Add, new Output(1), new Div, new Abs, 
            new IncI, new IncV, new Output(2), new Input(0), new Output(2), new Output(2), new Div, new Output(1), 
            new EndGoTo, new IncV, new JumpIfN, new Add, new EndJump, new SetI(2), new Add, new EndJump, 
            new EndGoTo, new IncV, new EndGoTo, new JumpIfN, new DecV, new SetI(2), new Div, new SetI(3), 
            new Add, new IncI, new EndJump, new EndGoTo, new Store, new Output(0), new SetI(2), new ItoV, 
            new EndJump, new Div, new IncI, new Output(2), new SetI(3), new Input(2), new Load, new Cmp, 
            new SetI(4), new Input(2), new SetI(2), new IfP, new Output(3), new IncI, new Output(2), new IncI, 
            new Output(2), new Input(0), new Output(2), new IncV, new Input(0), new Output(2), new VtoI, new Input(0), 
            new Div, new Input(0), new Neg, new Input(0), new Div, new Output(1), new IncV, new JumpIfN, 
            new SetI(3), new Input(2), new Cmp, new EndJump, new Input(2), new Div, new Input(1), new Sub, 
            new JumpIfN, new EndJump, new SetI(2), new Div, new IfP, new IncI, new IncV, new Input(0), 
            new Output(2), new EndJump, new GoToIfP, new Input(0), new Div, new SetI(4), new EndGoTo, new Cmp, 
            new IncV, new JumpIfN, new EndJump, new Output(3), new SetI(2), new Input(2), new SetI(2), new IncI, 
            new IncI, new Swap, new Neg, new Output(3), new Div, new Store, new Add, new EndJump, 
            new SetI(3), new Add, new Input(1), new Sub, new GoToIfP, new Input(2), new SetI(4), new EndJump, 
            new GoToIfP, new SetI(2), new IncI, new Output(2), new Store, new Output(2), new Input(0), new Store, 
            new Output(1), new Output(2), new DecV, new EndJump, new Store, new GoToIfP, new Sub, new Sub, 
            new Sub, new Input(3), new IfN, new EndGoTo, new SetI(3), new Swap, new Abs, new Input(0), 
            new Output(2), new Div, new EndGoTo, new JumpIfN, new SetI(2), new IfP, new IncV, new IncI, 
            new Div, new Input(0), new Output(2), new GoToIfP, new EndJump, new Store, new IfN, new Sub, 
            new Sub, new Input(3), new Output(0), new EndGoTo, new SetI(3), new Abs, new Input(0), new Output(2), 
            new Div, new EndJump, new Output(1), new Output(2), new Div, new EndJump, new Store, new SetI(2), 
            new Input(0), new Swap, new Div, new EndJump, new Output(1), new SetI(3), new Input(2), new Add, 
            new EndGoTo, new Input(2), new SetI(4), new Sub, new GoToIfP, new Input(2), new Input(0), new EndJump, 
            new Div, new Output(1), new EndGoTo, new IncI, new Add, new Neg, new Output(3), new Add, 
            new EndJump, new Add, new SetI(3), new Input(1), new Sub, new GoToIfP, new Input(2), new SetI(4), 
            new EndJump, new Input(3), new Input(3), new GoToIfP, new Load, new Store, new Sub, new SetI(3), 
            new IncI, new Input(0), new SetI(4), new IncV, new JumpIfN, new Cmp, new IncV, new EndJump, 
            new Input(2), new SetI(4), new EndGoTo, new ItoV, new JumpIfN, new EndJump, new SetI(2), new Add, 
            new Output(0), new SetI(3), new Input(2), new Cmp, new EndJump, new SetI(2), new IfP, new SetI(1), 
            new IncI, new Output(2), new Load, new Input(0), new Output(2), new IfN, new Output(1), new EndGoTo, 
            new IncV, new JumpIfN, new Store, new SetI(2), new IfP, new IncV, new IncI, new Div, 
            new Input(0), new Output(2), new GoToIfP, new EndJump, new Store, new IfN, new Sub, new Sub, 
            new Input(3), new Output(0), new EndGoTo, new SetI(3), new JumpIfN, new Add, new Div, new SetI(2), 
            new Div, new IfP, new IncI, new Output(2), new Output(2), new Input(0), new Input(0), new Output(2), 
            new Input(0), new Div, new Mul, new SetI(2), new Output(1), new EndGoTo, new Output(3), new Neg, 
            new Output(3), new Add, new EndJump, new SetI(3), new Add, new Input(1), new Sub, new GoToIfP, 
            new Input(2), new SetI(4), new Neg, new Load, new GoToIfP, new Output(3), new Sub, new SetI(3), 
            new Input(0), new IncV, new GoToIfP, new SetI(4), new Input(2), new GoToIfP, new EndJump, new Load, 
            new Sub, new SetI(3), new IncI, new Input(0), new Input(0), new IncV, new JumpIfN, new Div, 
            new EndJump, new Output(0), new SetI(4), new Input(2), new SetI(2), new IncI, new DecI, new IncV, 
            new Add, new Input(1), new Sub, new GoToIfP, new Input(2), new SetI(4), new JumpIfN, new SetI(2), 
            new Add, new EndGoTo, new IfP, new SetI(1), new IncI, new Output(2), new Load, new Input(0), 
            new Output(2), new Input(0), new Output(1), new EndGoTo, new IncV, new JumpIfN, new Store, new SetI(2), 
            new IfP, new IncV, new IncI, new Div, new Input(0), new Output(2), new GoToIfP, new EndJump, 
            new Store, new IfN, new Sub, new Sub, new Input(3), new Output(0), new EndGoTo, new SetI(3), 
            new Abs, new Input(0), new Output(2), new Div, new EndJump, new Output(1), new IfP, new SetI(3), 
            new Input(2), new Add, new EndGoTo, new Input(2), new EndGoTo, new EndJump, new Output(2), new DecV, 
            new Output(0), new GoToIfP, new Store, new Sub, new SetI(4), new IncV, new Input(0), new Output(2), 
            new Input(0), new Div, new Output(1), new Add, new EndGoTo, new IncV, new Output(2), new EndGoTo, 
            new JumpIfN, new SetI(2), new EndJump, new Div, new ItoV, new SetI(3), new Input(2), new EndJump, 
            new SetI(2), new Input(3), new EndJump, new Add, new Div, new IfP, new Sub, new Add, 
            new IfP, new Div, new IncI, new Output(1), new DecI, new Cmp, new Input(2), new EndJump, 
            new Input(2), new SetI(4), new IfP, new SetI(2), new VtoI, new Output(2), new Div, new IncI, 
            new Input(0), 
        ];
        this.jumpTable = [
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            29, null, null, null, null, 24, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, 57, null, null, null, null, 
            null, 51, null, null, null, 76, 123, null, 
            79, null, null, null, null, null, null, 72, 
            71, null, null, null, 61, null, null, 64, 
            null, null, null, null, null, null, null, null, 
            91, null, null, 88, null, null, null, null, 
            null, null, null, null, null, null, 134, 106, 
            null, null, 103, null, null, null, null, null, 
            null, null, null, null, null, null, 130, null, 
            null, null, null, 62, null, null, null, null, 
            null, null, 118, null, null, null, 102, null, 
            null, null, null, null, null, null, null, null, 
            148, null, null, null, 144, null, null, null, 
            null, null, null, 156, 155, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            311, null, 179, 178, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            214, null, null, null, null, null, 208, 289, 
            null, 278, 262, 226, null, null, null, null, 
            null, null, 219, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            272, 243, null, 241, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            268, null, null, null, null, null, 218, null, 
            null, null, null, null, 256, null, null, null, 
            240, null, null, null, null, null, 217, null, 
            null, null, null, null, null, null, null, null, 
            null, 215, null, null, null, null, null, 307, 
            null, null, null, null, null, null, null, null, 
            null, null, null, 295, null, null, null, 176, 
            null, null, null, null, null, null, null, null, 
            null, null, null, 343, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            651, null, 340, null, 338, null, null, 323, 
            453, null, 440, 354, null, null, null, null, 
            null, null, 347, 410, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, 395, 
            null, null, null, 391, null, null, null, null, 
            401, 400, null, null, null, null, null, null, 
            null, null, 355, null, null, null, 436, null, 
            null, 418, 417, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, 414, null, null, null, 
            346, null, null, null, null, null, null, null, 
            null, null, null, null, null, 344, null, null, 
            null, null, null, 629, null, null, null, null, 
            null, null, 475, 476, null, null, null, null, 
            null, null, null, 466, 467, null, null, null, 
            null, null, null, 531, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            508, null, null, null, 504, null, null, null, 
            null, null, 525, null, null, null, null, null, 
            null, null, null, null, null, 514, null, null, 
            null, null, null, 483, null, null, null, null, 
            null, null, null, null, 543, null, null, 540, 
            null, null, 626, null, 549, 548, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, 578, 
            null, 579, null, null, null, null, null, null, 
            null, null, 567, 569, null, null, null, null, 
            null, null, 620, null, 610, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, 615, null, null, 
            null, null, 588, null, null, null, null, 605, 
            null, null, null, null, 586, null, null, null, 
            null, null, 546, null, null, 459, null, null, 
            null, null, null, null, null, null, 640, null, 
            638, null, null, null, null, null, null, null, 
            null, null, null, 336, null, null, 692, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, 678, null, 679, null, null, 
            null, null, null, null, null, null, 667, 669, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, 654, null, null, null, 
            null, null, null, null, 705, null, null, null, 
            null, 700, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            722, null, 720, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, null, null, null, null, null, null, null, 
            null, 
        ];
        this.data = [
            0.0, 0.0, 0.0, 0.0, 
        ];


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
