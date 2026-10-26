use embedded_hal::digital::OutputPin;

pub const NUM: usize = 1; // modules in the chain

pub struct Max7219<DIN, CLK, CS> {
    pub din: DIN,
    pub clk: CLK,
    pub cs: CS,
    row: u8,
    col: u8,
    current_module: u8,
    buf: [[u8; 8]; 4],
    current_dot: (u8, u8),
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
            buf: [
                [1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
                [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
                [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
                [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
            ],
            current_dot: (0, 0),
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

    pub fn show_buf(&mut self) {
        let buf = self.buf;
        for row_index in 0..8 {
            self.cs.set_low().ok();

            for module_index in (0..4).rev() {
                // Send the row address (MAX7219 rows are addressed 1–8)
                self.shift_byte((row_index + 1) as u8);

                // Send the row's LED bit pattern
                self.shift_byte(buf[module_index][row_index]);
            }
            self.cs.set_high().ok();
        }
    }

    //second implementation

    pub fn update_buf_move_up(&mut self) {
        let module = self.current_dot.0;
        let row = self.current_dot.1;
        let bit = self.buf[module as usize][row as usize];

        if row == 7 {
            println!("Top end reached");
            return;
        }

        println!("UP -> before update current dot:{:?}", self.current_dot);
        println!("UP -> bit value:{:?}", bit);
        self.buf[module as usize][row as usize] = 0u8;

        self.current_dot.1 = self.current_dot.1 + 1;
        println!("UP -> before update current dot:{:?}", self.current_dot);
        self.buf[module as usize][self.current_dot.1 as usize] = bit;
        println!("UP -> new buf:{:?}", self.buf);

        self.show_buf();
    }

    pub fn update_buf_move_down(&mut self) {
        let module = self.current_dot.0;
        let row = self.current_dot.1;
        let bit = self.buf[module as usize][row as usize];

        if row == 0 {
            println!("Bottom end reached");
            return;
        }

        self.buf[module as usize][row as usize] = 0u8;

        self.current_dot.1 = self.current_dot.1 - 1;
        self.buf[module as usize][self.current_dot.1 as usize] = bit;

        self.show_buf();
    }

    pub fn update_buf_move_right(&mut self) {
        let mut module = self.current_dot.0;
        let row = self.current_dot.1;
        let bit = self.buf[module as usize][row as usize];
        let mut new_bit = self.buf[module as usize][row as usize] << 1;

        println!("RIGHT -> bit first:{:?}", bit);
        if bit == 128 {
            if self.current_dot.0 == 3 {
                println!("x axis right end reached");
                return;
            } else {
                module += 1;
                new_bit = 1u8;
                //reset prev module
                let prev_module = module - 1;
                self.buf[prev_module as usize][row as usize] = 0u8;
                self.current_dot.0 = module;
            }
        }

        println!("RIGHT -> before current dot:{:?}", self.current_dot);
        println!("RIGHT -> new bit value:{:?}", new_bit);
        self.buf[module as usize][row as usize] = new_bit;

        println!("RIGHT -> after current dot:{:?}", self.current_dot);
        println!("RIGHT -> new buf:{:?}", self.buf);
        // self.current_dot.1 = self.current_dot.1 - 1;
        // self.buf[module as usize][self.current_dot.1 as usize] = 1u8;

        self.show_buf();
    }

    pub fn update_buf_move_left(&mut self) {
        let mut module = self.current_dot.0;
        let row = self.current_dot.1;
        let bit = self.buf[module as usize][row as usize];
        let mut new_bit = self.buf[module as usize][row as usize] >> 1;

        println!("LEFT -> bit first:{:?}", bit);
        if bit == 1 {
            if self.current_dot.0 == 0 {
                println!("x axis left end reached");
                return;
            } else {
                module -= 1;
                new_bit = 1u8 << 7;
                //reset prev module
                let prev_module = module + 1;
                self.buf[prev_module as usize][row as usize] = 0u8;
                self.current_dot.0 = module;
            }
        }
        println!("RIGHT -> before current dot:{:?}", self.current_dot);
        println!("RIGHT -> new bit value:{:?}", new_bit);
        self.buf[module as usize][row as usize] = new_bit;

        println!("RIGHT -> after current dot:{:?}", self.current_dot);
        println!("RIGHT -> new buf:{:?}", self.buf);
        // self.current_dot.1 = self.current_dot.1 - 1;
        // self.buf[module as usize][self.current_dot.1 as usize] = 1u8;

        self.show_buf();
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
        // self.show_one(self.row, bit);
        self.show_buf();
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

    //first implementation

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
