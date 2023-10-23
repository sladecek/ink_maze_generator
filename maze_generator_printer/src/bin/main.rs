use std::{env, fs, io};
use regex::Regex;

use maze_generator_printer::{
    svg_painter::paint_shapes,
    rectangular_builder::Builder,
};


fn main() {
    let width: i32 = env::args().nth(1).unwrap().parse().unwrap();
    let height: i32 = env::args().nth(2).unwrap().parse().unwrap();
    let mut instance: Vec<bool> = vec![];
    let re = Regex::new(r" *Data Ok\(\[([0-9 ,]+)").unwrap();
    for line in io::stdin().lines() {
        let l = line.unwrap();
        let m = re.captures(&l);
        if m.is_some() {
            for r in m.unwrap().get(1).unwrap().as_str().split(",") {
                let bits: u8 = r.trim().parse().unwrap();
                //println!("{}", bits);
                for b in 0..8 {
                    let v: bool = bits & (1 << b) != 0;
                    //println!("  {}", v);
                    instance.push(v);
                }
            }
        }
    }
    //println!("{:?}", instance);

    let builder = Builder::new(width, height);
    let shapes = builder.build();
    let svg = paint_shapes(&shapes, &instance, 0, width * height - 1);
    let pdf = svg2pdf::convert_str(&svg, svg2pdf::Options::default()).unwrap();
    fs::write("maze.pdf", pdf).unwrap();
}

