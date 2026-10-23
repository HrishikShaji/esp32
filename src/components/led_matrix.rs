use embedded_hal::digital::OutputPin;

pub const NUM: usize = 1; // modules in the chain

pub struct Max7219<DIN, CLK, CS> {
    pub din: DIN,
    pub clk: CLK,
    pub cs: CS,
    row: u8,
    col: u8,
    current_module: u8,
}

impl<DIN: OutputPin, CLK: OutputPin, CS: OutputPin> Max7219<DIN, CLK, CS> {
    pub fn new(din: DIN, clk: CLK, cs: CS) -> Self {
        Self {
            din,
            clk,
            cs,
            row: 1,
            col: 0,
            current_module: 3,
        }
    }

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
        let bit = 1u8 << self.col;
        self.show_one(self.row, bit);
    }

    fn clear(&mut self) {
        for row in 1..=8 {
            self.write_all(row, 0x00);
        }
    }

    pub fn show_one(&mut self, row: u8, bit: u8) {
        self.cs.set_low().ok();

        // Send the command for the farthest module first
        // for _ in 0..3 {
        //     self.shift_byte(1);
        //     self.shift_byte(0b00000000);
        // }

        // Send the command for the nearest module last

        for i in 0..4 {
            self.shift_byte(row);
            if i == self.current_module {
                self.shift_byte(bit);
            } else {
                self.shift_byte(0u8);
            }
        }

        self.cs.set_high().ok();
    }

    pub fn lit_row(mut self) {
        for row in 0..8 {
            self.show_one(row, 0b00000001);
        }
    }

    pub fn move_up(&mut self) {
        println!("UP button clicked");
        if self.row < 8 {
            self.show_one(self.row, 0b00000000);
            self.row = self.row + 1;
            let bit = 1u8 << self.col;
            println!("Row number {}", self.row);
            self.show_one(self.row, bit);
        } else {
            println!("Row exceeded");
        }
    }
    pub fn move_down(&mut self) {
        println!("DOWN button clicked");

        if self.row > 1 {
            self.show_one(self.row, 0b00000000);
            self.row = self.row - 1;
            let bit = 1u8 << self.col;
            self.show_one(self.row, bit);
        } else {
            println!("0 index");
        }
    }
    pub fn move_left(&mut self) {
        println!("LEFT button clicked");
        if self.col > 0 {
            self.col = self.col - 1;
            let bit = 1u8 << self.col;
            self.show_one(self.row, bit);
        } else {
            if self.current_module == 3 {
                println!("0 index");
            } else {
                self.current_module = self.current_module + 1;
                self.col = 7;
                let bit = 1u8 << self.col;
                println!("bit is {}", bit);
                self.show_one(self.row, bit);
            }
        }
    }
    pub fn move_right(&mut self) {
        println!("RIGHT button clicked");

        if self.col < 7 {
            self.col = self.col + 1;
            let bit = 1u8 << self.col;
            println!("bit is {}", bit);
            self.show_one(self.row, bit);
        } else {
            if self.current_module == 0 {
                println!("Col exceeded");
            } else {
                self.current_module = self.current_module - 1;
                self.col = 0;
                let bit = 1u8 << self.col;
                println!("bit is {}", bit);
                self.show_one(self.row, bit);
            }
        }
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
