use std::fs;

use rand::random;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 384;

struct Chip8 {
    memory: [u8; 4096],                           // メモリ
    registers: [u8; 16],                          // レジスタ
    stack: [u16; 16],                             // スタック
    i: u16,                                       // インデックスレジスタ
    pc: u16,                                      // プログラムカウンタ
    sp: u8,                                       // スタックポインタ
    delay: u8,                                    // ディレイタイマ
    sound: u8,                                    // サウンドタイマ
    keyboard: [bool; 16],                         // キー入力状態
    display: [[u8; SCREEN_HEIGHT]; SCREEN_WIDTH], // ディスプレイ
}

impl Chip8 {
    // 初期化
    fn new() -> Chip8 {
        // メモリとレジスタを初期化
        let mut memory = [0; 4096];
        let registers = [0; 16];
        let stack = [0; 16];

        // メモリの先頭から順に，フォントセットをロード
        for i in 0..80 {
            memory[i] = FONTSET[i];
        }

        Chip8 {
            memory,
            registers,
            stack,
            i: 0,
            pc: 0x200,
            sp: 0,
            delay: 0,
            sound: 0,
            keyboard: [false; 16],
            display: [[0; SCREEN_HEIGHT]; SCREEN_WIDTH],
        }
    }

    // ゲームプログラムの実行
    fn run(&mut self, program: &[u8]) {
        // メモリの先頭から順に，ゲームプログラムをロード
        for (i, &byte) in program.iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }

        // メインループ
        loop {
            // 命令を取得し，実行
            let opcode = (self.memory[self.pc as usize] as u16) << 8
                | self.memory[self.pc as usize + 1] as u16;
            self.execute_opcode(opcode);

            // タイマーの更新
            if self.delay > 0 {
                self.delay -= 1;
            }
            if self.sound > 0 {
                self.sound -= 1;
                if self.sound == 0 {
                    // サウンドの再生
                }
            }
        }
    }

    // 命令の実行
    fn execute_opcode(&mut self, opcode: u16) {
        // opcodeの上位8ビットを取得
        let x = ((opcode & 0x0F00) >> 8) as usize;
        // opcodeの下位8ビットを取得
        let y = ((opcode & 0x00F0) >> 4) as usize;
        // opcodeの下位4ビットを取得
        let n = (opcode & 0x000F) as usize;
        // opcodeの下位12ビットを取得
        let nnn = (opcode & 0x0FFF) as u16;
        // opcodeの下位8ビットを取得
        let kk = (opcode & 0x00FF) as u8;

        // 各命令に応じた処理
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.cls(), // 00E0 - CLS
                0x00EE => self.ret(), // 00EE - RET
                _ => panic!("Unknown opcode: {:X}", opcode),
            },
            0x1000 => self.jp(nnn),        // 1NNN - JP addr
            0x2000 => self.call(nnn),      // 2NNN - CALL addr
            0x3000 => self.se(x, kk),      // 3XKK - SE Vx, byte
            0x4000 => self.sne(x, kk),     // 4XKK - SNE Vx, byte
            0x5000 => self.se_vx_vy(x, y), // 5XY0 - SE Vx, Vy
            0x6000 => self.ld(x, kk),      // 6XKK - LD Vx, byte
            0x7000 => self.add(x, kk),     // 7XKK - ADD Vx, byte
            0x8000 => match opcode & 0x000F {
                0x0000 => self.ld_vx_vy(x, y),  // 8XY0 - LD Vx, Vy
                0x0001 => self.or(x, y),        // 8XY1 - OR Vx, Vy
                0x0002 => self.and(x, y),       // 8XY2 - AND Vx, Vy
                0x0003 => self.xor(x, y),       // 8XY3 - XOR Vx, Vy
                0x0004 => self.add_vx_vy(x, y), // 8XY4 - ADD Vx, Vy
                0x0005 => self.sub(x, y),       // 8XY5 - SUB Vx, Vy
                0x0006 => self.shr(x),          // 8XY6 - SHR Vx
                0x0007 => self.subn(x, y),      // 8XY7 - SUBN Vx, Vy
                0x000E => self.shl(x),          // 8XYE - SHL Vx
                _ => panic!("Unknown opcode: {:X}", opcode),
            },
            0x9000 => self.sne_vx_vy(x, y), // 9XY0 - SNE Vx, Vy
            0xA000 => self.ld_i(nnn),       // ANNN - LD I, addr
            0xB000 => self.jp_v0(nnn),      // BNNN - JP V0, addr
            0xC000 => self.rnd(x, kk),      // CXKK - RND Vx, byte
            0xD000 => self.drw(x, y, n),    // DXYN - DRW Vx, Vy, nibble
            0xE000 => match opcode & 0x00FF {
                0x009E => self.skp(x),  // EX9E - SKP Vx
                0x00A1 => self.sknp(x), // EXA1 - SKNP Vx
                _ => panic!("Unknown opcode: {:X}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.ld_vx_dt(x), // FX07 - LD Vx, DT
                0x000A => self.ld_vx_k(x),  // FX0A - LD Vx, K
                0x0015 => self.ld_dt_vx(x), // FX15 - LD DT, Vx
                0x0018 => self.ld_st_vx(x), // FX18 - LD ST, Vx
                0x001E => self.add_i_vx(x), // FX1E - ADD I, Vx
                0x0029 => self.ld_f_vx(x),  // FX29 - LD F, Vx
                0x0033 => self.ld_b_vx(x),  // FX33 - LD B, Vx
                0x0055 => self.ld_i_vx(x),  // FX55 - LD [I], Vx
                0x0065 => self.ld_vx_i(x),  // FX65 - LD Vx, [I]
                _ => panic!("Unknown opcode: {:X}", opcode),
            },
            _ => panic!("Unknown opcode: {:X}", opcode),
        }
    }

    // 00E0 - CLS: 画面を消去
    fn cls(&mut self) {
        // 画面を消去する処理を実装する
    }

    // 00EE - RET: サブルーチンから復帰
    fn ret(&mut self) {
        // スタックからアドレスをポップし，プログラムカウンタをセットする
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }

    // 1NNN - JP addr: プログラムカウンタを指定されたアドレスへ移動
    fn jp(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    // 2NNN - CALL addr: サブルーチンを呼び出す
    fn call(&mut self, nnn: u16) {
        // 現在のプログラムカウンタをスタックにプッシュ
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        // プログラムカウンタを指定されたアドレスへ移動
        self.pc = nnn;
    }

    // 3XKK - SE Vx, byte: Vxと指定された値が等しい場合，プログラムカウンタを2つ進める
    fn se(&mut self, x: usize, kk: u8) {
        if self.registers[x] == kk {
            self.pc += 2;
        }
    }

    // 4XKK - SNE Vx, byte: Vxと指定された値が等しくない場合，プログラムカウンタを2つ進める
    fn sne(&mut self, x: usize, kk: u8) {
        if self.registers[x] != kk {
            self.pc += 2;
        }
    }

    // 5XY0 - SE Vx, Vy: VxとVyが等しい場合，プログラムカウンタを2つ進める
    fn se_vx_vy(&mut self, x: usize, y: usize) {
        if self.registers[x] == self.registers[y] {
            self.pc += 2;
        }
    }

    // 6XKK - LD Vx, byte: Vxに指定された値を代入する
    fn ld(&mut self, x: usize, kk: u8) {
        self.registers[x] = kk;
    }

    // 7XKK - ADD Vx, byte: Vxに指定された値を加える
    fn add(&mut self, x: usize, kk: u8) {
        self.registers[x] = self.registers[x].wrapping_add(kk);
    }

    // 8XY0 - LD Vx, Vy: VxにVyを代入する
    fn ld_vx_vy(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
    }

    // 8XY1 - OR Vx, Vy: VxにVx OR Vyを代入する
    fn or(&mut self, x: usize, y: usize) {
        self.registers[x] |= self.registers[y];
    }

    // 8XY2 - AND Vx, Vy: VxにVx AND Vyを代入する
    fn and(&mut self, x: usize, y: usize) {
        self.registers[x] &= self.registers[y];
    }

    // 8XY3 - XOR Vx, Vy: VxにVx XOR Vyを代入する
    fn xor(&mut self, x: usize, y: usize) {
        self.registers[x] ^= self.registers[y];
    }

    // 8XY4 - ADD Vx, Vy: VxにVx + Vyを代入する
    fn add_vx_vy(&mut self, x: usize, y: usize) {
        let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = if overflow { 1 } else { 0 };
    }

    // 8XY5 - SUB Vx, Vy: VxからVyを引いた値をVxに代入する
    fn sub(&mut self, x: usize, y: usize) {
        self.registers[0xF] = if self.registers[x] > self.registers[y] {
            1
        } else {
            0
        };
        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
    }

    // 8XY6 - SHR Vx: Vxの右ビットをVxに代入し，VFにVxの最下位ビットを代入する
    fn shr(&mut self, x: usize) {
        self.registers[0xF] = self.registers[x] & 0x01;
        self.registers[x] >>= 1;
    }

    // 8XY7 - SUBN Vx, Vy: VyからVxを引いた値をVxに代入する
    fn subn(&mut self, x: usize, y: usize) {
        self.registers[0xF] = if self.registers[y] > self.registers[x] {
            1
        } else {
            0
        };
        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
    }

    // 8XYE - SHL Vx: Vxの左ビットをVxに代入し，VFにVxの最上位ビットを代入する
    fn shl(&mut self, x: usize) {
        self.registers[0xF] = (self.registers[x] & 0x80) >> 7;
        self.registers[x] <<= 1;
    }

    // 9XY0 - SNE Vx, Vy: VxとVyが等しくない場合，プログラムカウンタを2つ進める
    fn sne_vx_vy(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.pc += 2;
        }
    }

    // ANNN - LD I, addr: インデックスレジスタに指定された値を代入する
    fn ld_i(&mut self, nnn: u16) {
        self.i = nnn;
    }

    // BNNN - JP V0, addr: V0と指定された値を加えた値をプログラムカウンタに代入する
    fn jp_v0(&mut self, nnn: u16) {
        self.pc = self.registers[0] as u16 + nnn;
    }

    // CXKK - RND Vx, byte: 0から255までのランダムな値と指定された値をANDし，Vxに代入する
    fn rnd(&mut self, x: usize, kk: u8) {
        self.registers[x] = random::<u8>() & kk;
    }

    // DXYN - DRW Vx, Vy, nibble: Vx, Vyからインデックスレジスタに保持されたアドレスからnibble個分のデータを取得し，画面上に描画する
    fn drw(&mut self, x: usize, y: usize, n: usize) {
        // Vx, Vyから座標を取得する
        let x = self.registers[x] as usize;
        let y = self.registers[y] as usize;

        // スプライトを描画する
        let mut collision = false;
        for i in 0..n {
            let sprite_line = self.memory[self.i as usize + i];

            for j in 0..8 {
                let sprite_pixel = (sprite_line >> (7 - j)) & 0x01;
                let screen_x = (x + j) % SCREEN_WIDTH;
                let screen_y = (y + i as usize) % SCREEN_HEIGHT;

                let screen_pixel = self.display[screen_y][screen_x];
                collision |= screen_pixel == 1 && sprite_pixel == 1;
                self.display[screen_y][screen_x] ^= sprite_pixel;
            }
        }

        // 衝突が発生したかどうかをVFに代入する
        self.registers[0xF] = if collision { 1 } else { 0 };
    }

    // EX9E - SKP Vx: キーボードのVx番目のキーが押されている場合，プログラムカウンタを2つ進める
    fn skp(&mut self, x: usize) {
        if self.keyboard[self.registers[x] as usize] {
            self.pc += 2;
        }
    }

    // EXA1 - SKNP Vx: キーボードのVx番目のキーが押されていない場合，プログラムカウンタを2つ進める
    fn sknp(&mut self, x: usize) {
        if !self.keyboard[self.registers[x] as usize] {
            self.pc += 2;
        }
    }

    // FX07 - LD Vx, DT: Vxにデルタタイムを代入する
    fn ld_vx_dt(&mut self, x: usize) {
        self.registers[x] = self.delay;
    }

    // FX0A - LD Vx, K: キー入力を待つ
    fn ld_vx_k(&mut self, x: usize) {
        // ボタンが押されるまで待つ
        loop {
            let button_pressed = self.keyboard.iter().position(|&b| b);
            if let Some(i) = button_pressed {
                self.registers[x] = i as u8;
                break;
            }
        }
    }

    // FX15 - LD DT, Vx: デルタタイムにVxを代入する
    fn ld_dt_vx(&mut self, x: usize) {
        self.delay = self.registers[x];
    }

    // FX18 - LD ST, Vx: サウンドタイマにVxを代入する
    fn ld_st_vx(&mut self, x: usize) {
        self.sound = self.registers[x];
    }

    // FX1E - ADD I, Vx: インデックスレジスタにVxを加える
    fn add_i_vx(&mut self, x: usize) {
        self.i += self.registers[x] as u16;
    }

    // FX29 - LD F, Vx: インデックスレジスタにVx番目のフォントを代入する
    fn ld_f_vx(&mut self, x: usize) {
        self.i = (self.registers[x] as usize * 5) as u16;
    }

    // FX33 - LD B, Vx: インデックスレジスタにVxを十進数表記で代入する
    fn ld_b_vx(&mut self, x: usize) {
        let value = self.registers[x];
        self.memory[self.i as usize] = value / 100;
        self.memory[(self.i + 1) as usize] = (value / 10) % 10;
        self.memory[(self.i + 2) as usize] = value % 10;
    }

    // FX55 - LD [I], Vx: インデックスレジスタからV0からVxまでのレジスタの値を順番に保存する
    fn ld_i_vx(&mut self, x: usize) {
        for i in 0..=x {
            self.memory[self.i as usize + i] = self.registers[i];
        }
    }

    // FX65 - LD Vx, [I]: インデックスレジスタからV0からVxまでのレジスタに順番に値を代入する
    fn ld_vx_i(&mut self, x: usize) {
        for i in 0..=x {
            self.registers[i] = self.memory[self.i as usize + i];
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = fs::read(args[1].to_string());

    match result {
        Ok(file) => {
            let mut chip8 = Chip8::new();
            chip8.run(&file);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
