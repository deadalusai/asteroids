use bevy_prototype_lyon::prelude::*;

type BoxError = Box<dyn std::error::Error + 'static>;

enum Instruction {
    MoveTo((f32, f32)),
    LineTo((f32, f32)),
    ClosePath,
}

fn parse_svg_instructions(path: &str) -> Result<Vec<Instruction>, BoxError> {
    let mut instructions = Vec::new();
    let mut tokens = path.split(" ").map(|s| s.trim()).filter(|s| s.len() > 0).into_iter();
    while let Some(token) = tokens.next() {
        instructions.push(match token {
            "M" => Instruction::MoveTo(parse_coordinates(tokens.next(), tokens.next())?),
            "L" => Instruction::LineTo(parse_coordinates(tokens.next(), tokens.next())?),
            "Z" => Instruction::ClosePath,
            _ => return Err("Unknown instruction".into())
        });
    }
    Ok(instructions)
}

fn parse_coordinates(a: Option<&str>, b: Option<&str>) -> Result<(f32, f32), BoxError> {
    let a = a.ok_or("Expected first coordinate")?.parse::<f32>()?;
    let b = b.ok_or("Expected second coordinate")?.parse::<f32>()?;
    Ok((a, b))
}

fn svg_to_bevy((x, y): (f32, f32)) -> bevy::prelude::Vec2 {
    // NOTE: SVG uses {x right, y down}, bevy uses {x right, y up}
    // Flip the y coordinates
    bevy::prelude::Vec2::new(x, -y)
}

pub fn simple_svg_to_path(path: &str) -> Path {
    let mut p = PathBuilder::new();
    for instruction in parse_svg_instructions(path).unwrap() {
        match instruction {
            Instruction::MoveTo(coords) => {
                p.move_to(svg_to_bevy(coords));
            },
            Instruction::LineTo(coords) => {
                p.line_to(svg_to_bevy(coords));
            },
            Instruction::ClosePath => {
                p.close();
            },
        }
    }
    p.build()
}