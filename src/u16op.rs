use crate::ops;

pub struct Parser {
    pub code:     u16,
    pub position: i32,
}


impl Parser {
    pub fn unsigned(&mut self, bit_size: i32) -> u16 {
        if self.position - bit_size < 0 {
            panic!("u16 op reader overflow: code={}, read_bits={}", self.code, 16 - self.position + bit_size);
        }
        self.position -= bit_size;
        return (self.code >> self.position) & ((1 << bit_size) - 1);
    }
    pub fn fixed(&mut self, bit_size: i32, expected: u16) -> Result<(), ops::OpParseError> {
        let actual = self.unsigned(bit_size);
        return if actual == expected { Ok(()) } else { Err(ops::OpParseError::FixedMismatch { code: self.code, segment: self.position..self.position + bit_size, expected, actual }) };
    }
    pub fn register(&mut self) -> ops::Register {
        return ops::Register(self.unsigned(3) as usize);
    }

    pub fn signed(&mut self, bit_size: i32) -> u16 {
        let value = self.unsigned(bit_size);
        let sign = value >> (bit_size - 1);
        return if sign == 1 { value | (u16::MAX << bit_size) } else { value };
    }
    pub fn argument(&mut self) -> Result<ops::Argument, ops::OpParseError> {
        match self.unsigned(1) {
            1 => Ok(ops::Argument::Immediate(self.signed(5))),
            _ => {
                self.fixed(2, 0b00)?;
                Ok(ops::Argument::Register(self.register()))
            }
        }
    }
}

impl ops::Operation {
    /// spec: https://www.jmeiners.com/lc3-vm/supplies/lc3-isa.pdf
    pub fn parse(code: u16) -> Result<Self, ops::OpParseError> {
        let mut parser = Parser { code, position: 16 };
        return match parser.unsigned(4) {
            0b0001 => {
                let dr = parser.register();
                let sr1 = parser.register();
                let arg = parser.argument()?;
                Ok(ops::Operation::OpAdd { dr, sr1, arg })
            }
            0b0101 => {
                let dr = parser.register();
                let sr1 = parser.register();
                let arg = parser.argument()?;
                Ok(ops::Operation::OpAnd { dr, sr1, arg })
            }
            0b0000 => {
                let n = parser.unsigned(1) == 1;
                let z = parser.unsigned(1) == 1;
                let p = parser.unsigned(1) == 1;
                let pc_offset = parser.signed(9);
                Ok(ops::Operation::OpBr { n, z, p, pc_offset })
            }
            0b1100 => {
                parser.fixed(3, 0)?;
                let base_r = parser.register();
                parser.fixed(6, 0)?;
                Ok(ops::Operation::OpJmp { base_r })
            }
            0b0100 => match parser.unsigned(1) {
                1 => {
                    let pc_offset = parser.signed(11);
                    Ok(ops::Operation::OpJsr { pc_offset })
                }
                _ => {
                    parser.fixed(2, 0)?;
                    let base_r = parser.register();
                    parser.fixed(6, 0)?;
                    Ok(ops::Operation::OpJsrr { base_r })
                }
            },
            0b0010 => {
                let dr = parser.register();
                let pc_offset = parser.signed(9);
                Ok(ops::Operation::OpLd { dr, pc_offset })
            }
            0b1010 => {
                let dr = parser.register();
                let pc_offset = parser.signed(9);
                Ok(ops::Operation::OpLdi { dr, pc_offset })
            }
            0b0110 => {
                let dr = parser.register();
                let base_r = parser.register();
                let offset = parser.signed(6);
                Ok(ops::Operation::OpLdr { dr, base_r, offset })
            }
            0b1110 => {
                let dr = parser.register();
                let pc_offset = parser.signed(9);
                Ok(ops::Operation::OpLea { dr, pc_offset })
            }
            0b1001 => {
                let dr = parser.register();
                let sr = parser.register();
                parser.fixed(6, 0b111111)?;
                Ok(ops::Operation::OpNot { dr, sr })
            }
            0b1000 => {
                parser.fixed(12, 0)?;
                Ok(ops::Operation::OpRti)
            }
            0b0011 => {
                let sr = parser.register();
                let pc_offset = parser.signed(9);
                Ok(ops::Operation::OpSt { sr, pc_offset })
            }
            0b1011 => {
                let sr = parser.register();
                let pc_offset = parser.signed(9);
                Ok(ops::Operation::OpSti { sr, pc_offset })
            }
            0b0111 => {
                let sr = parser.register();
                let base_r = parser.register();
                let offset = parser.signed(6);
                Ok(ops::Operation::OpStr { sr, base_r, offset })
            }
            0b1111 => {
                parser.fixed(4, 0)?;
                let trap_vector = parser.unsigned(8);
                Ok(ops::Operation::OpTrap { trap_vector })
            }
            0b1101 => Err(ops::OpParseError::IllegalOpcode { code }),
            _ => unreachable!("all op code prefixes must be covered"),
        };
    }
}
