/* Descriptions of parts, chips and signal values. */


export const CHIP_FN = {
  not: 'Y = NOT A', nand: 'Y = NOT (A AND B)', nor: 'Y = NOT (A OR B)', and: 'Y = A AND B', or: 'Y = A OR B',
  xor: 'Y = 1 when A and B differ', mux: 'Y = A when SEL is 0, B when SEL is 1', half_adder: 'Adds two bits. S is the sum, C the carry.',
  full_adder: 'Adds A + B + CIN.', add4: 'Adds two 4-bit numbers with carry.', add8: 'Adds two bytes with carry.',
  flip4: 'Inverts X when INV is 1.', flip8: 'Inverts an 8-bit bus when INV is 1.', zero8: 'Z is 1 when all 8 bits are 0.',
  alu8: 'S = A + B, or A - B when SUB is 1. C is carry, Z is zero.', sr_latch: 'S sets Q, R resets Q, both 0 holds.',
  d_latch: 'Q follows D while E is 1, holds when E is 0.', dff: 'Stores D on the rising edge of CLK.',
  reg_bit: 'Stores D on a clock edge when LOAD is 1.', reg4: 'Stores 4 bits on a clock edge when LOAD is 1.',
  reg8: 'Stores a byte on a clock edge when LOAD is 1.', accum: 'ACC = ACC +/- B on each clock edge.',
  inc4: 'Y = A + 1, wrapping 15 to 0.', mux4: 'Y = A when SEL is 0, B when SEL is 1 (4 bits).', mux8: 'Y = A when SEL is 0, B when SEL is 1 (8 bits).',
  pc4: 'Each clock edge: hold if EN is 0, else load D if LOAD, else count up.', dec2: 'Y[A] = EN, every other Y is 0.', dec4: 'Y[A] = EN for a 4-bit A.',
  ram2: '2 bytes of memory. Q shows the byte at A; writes D on a clock edge when WE.', ram4: '4 bytes of memory.', ram8: '8 bytes of memory.', ram16: '16 bytes of memory.',
  ctrl: 'Turns an opcode and the Z flag into control signals.', cpu: 'Executes the instruction on I each clock edge. PC says which instruction it wants.',
};

export const PART_INFO = {
  vcc: ['Supply rail', 'A constant 1: the positive supply. CMOS datasheets usually call it VDD; VCC is the older bipolar name.'],
  gnd: ['Ground rail', 'A constant 0. In real silicon, the 0 V reference.'],
  nmos: ['NMOS transistor', 'Connects S to D while G is 1. In real silicon it passes a strong 0 but only a weak, degraded 1.'],
  pmos: ['PMOS transistor', 'Connects S to D while G is 0 (the circle on G means inverted). In real silicon it passes a strong 1 but only a weak, degraded 0.'],
  rom: ['ROM', 'Program memory: 16 bytes. A0 to A3 pick an address, D0 to D7 show the byte stored there. Select it to write a program.'],
  display: ['Display', 'Shows its 8 input bits as a decimal number. B0 is the lowest bit.'],
};

export const SIG_INFO = {'1': 'charged (1)', '0': 'drained (0)', 'Z': 'floating: nothing drives it', 'X': 'unknown or conflicting'};
