use embedded_hal::digital::OutputPin;

pub const NUM: usize = 1; // modules in the chain

pub struct Max7219<DIN, CLK, CS> {
    pub din: DIN,
    pub clk: CLK,
    pub cs: CS,
}

impl<DIN: OutputPin, CLK: OutputPin, CS: OutputPin> Max7219<DIN, CLK, CS> {
    fn shift_byte(&mut self, b: u8) {
        for i in (0..8).rev() {
            if (b >> i) & 1 == 1 {
                self.din.set_high().ok();
            } else {
                self.din.set_low().ok();
            }
            self.clk.set_high().ok();
            self.clk.set_low().ok();
        }
    }

    // Write a register with a separate value for each module.
    // The first frame sent travels to the farthest module, so send in reverse.
    fn write_each(&mut self, reg: u8, data: [u8; NUM]) {
        self.cs.set_low().ok();
        for d in data.iter().rev() {
            self.shift_byte(reg);
            self.shift_byte(*d);
        }
        self.cs.set_high().ok();
    }

    fn write_all(&mut self, reg: u8, val: u8) {
        self.write_each(reg, [val; NUM]);
    }

    pub fn init(&mut self) {
        self.write_all(0x0F, 0x00); // display test off
        self.write_all(0x09, 0x00); // no BCD decode
        self.write_all(0x0B, 0x07); // scan all 8 rows
        self.write_all(0x0A, 0x03); // brightness 0..15
        self.write_all(0x0C, 0x01); // normal operation (wake up)
        self.clear();
    }

    fn clear(&mut self) {
        for row in 1..=8 {
            self.write_all(row, 0x00);
        }
    }

    pub fn show_one(&mut self, row: u8, bit: u8) {
        self.cs.set_low().ok();

        // Send the command for the farthest module first
        for _ in 0..3 {
            self.shift_byte(1);
            self.shift_byte(0b00000000);
        }

        // Send the command for the nearest module last
        self.shift_byte(row);
        self.shift_byte(bit);

        self.cs.set_high().ok();
    }

    pub fn move_left(&mut self, row: u8) {
        self.show_one(row, 0b00000001);
    }

    // buf[module][row]
    pub fn show(&mut self, buf: &[[u8; 8]; NUM]) {
        for row in 0..8 {
            let mut d = [0u8; NUM];
            for m in 0..NUM {
                d[m] = buf[m][row];
            }
            self.write_each(row as u8 + 1, d);
        }
    }
}
