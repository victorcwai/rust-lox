# Chunks of Bytecode #
## Content ##

### Bytecode ###
- Interpreter (Walk AST) vs Byte code vs Machine/Native code
- We compile our code into bytecode then run it on a VM
  - jlox parse into AST then execute the nodes in Interpreter

### Chunks of Instructions ###
- Each instruction has a one-byte **operation code** (universally shortened to **opcode**). 
- The VM executes according to the instruction.

#### Dynamic array of instructions ####
- Pros:
  - Cache-friendly, dense storage
  - Constant-time indexed element lookup
  - Constant-time appending to the end of the array
- When capacity < count, allocate a new array

### Disassembling Chunks ###
Given a blob of machine code, a **disassembler** spits out a textual listing of the instructions. We use that for debugging the instruction.

### Constants ###
#### Representing values ####
For small fixed-size values like integers, many instruction sets store the value directly in the code stream right after the opcode. These are called **immediate instructions** because the bits for the value are immediately after the opcode.

That doesn’t work well for large or variable-sized constants like strings. In a native compiler to machine code, those bigger constants get stored in a separate “constant data” region in the binary executable. Then, the instruction to load a constant has an address or offset pointing to where the value is stored in that section.

JVM uses constant pool with each compiled class. For rust-lox we do the same: Each chunk will carry with it a list of the values that appear as literals in the program. To keep things simpler, we’ll put all constants in there, even simple integers.

#### Constant instructions ####
**Operands** are stored as binary data immediately after the opcode in the instruction stream and let us parameterize what the instruction does.

Each opcode determines how many operand bytes it has and what they mean. Each time we add a new opcode to clox, we specify what its operands look like—its **instruction format**.

### Line Information ###
In the chunk, we store a separate array of integers that parallels the bytecode. Each number in the array is the line number for the corresponding byte in the bytecode. When a runtime error occurs, we look up the line number at the same index as the current instruction’s offset in the code array.

## Challenges (TODO) ##
## DESIGN NOTES - TEST YOUR LANGUAGE ##
TLDR: Write tests in your language! 

"Each test is a program written in the language along with the output or errors it is expected to produce. Then you have a test runner that pushes the test program through your language implementation and validates that it does what it’s supposed to."