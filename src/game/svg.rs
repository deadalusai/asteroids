use bevy_prototype_lyon::prelude::*;

type BoxError = Box<dyn std::error::Error + 'static>;

struct SvgPoint(f32, f32);

impl SvgPoint {
    fn to_vec2(&self) -> bevy::prelude::Vec2 {
        // NOTE: SVG uses {x right, y down}, bevy uses {x right, y up}
        // Flip the y coordinates
        bevy::prelude::Vec2::new(self.0, -self.1)
    }
}

struct SvgCubicBezierCurveTo {
    ctrl1: SvgPoint,
    ctrl2: SvgPoint,
    to: SvgPoint,
}

struct SvgQuadraticBezierCurveTo {
    ctrl: SvgPoint,
    to: SvgPoint,
}

enum Instruction {
    MoveTo(SvgPoint),
    LineTo(SvgPoint),
    CubicCurveTo(SvgCubicBezierCurveTo),
    QuadraticCurveTo(SvgQuadraticBezierCurveTo),
    ClosePath,
}

trait SvgParse {
    fn parse_point(&mut self) -> Result<SvgPoint, BoxError>;
    fn parse_cubic_curve_to(&mut self) -> Result<SvgCubicBezierCurveTo, BoxError>;
    fn parse_quadratic_curve_to(&mut self) -> Result<SvgQuadraticBezierCurveTo, BoxError>;
}

impl<'a, T: Iterator<Item=&'a str>> SvgParse for T {
    fn parse_point(&mut self) -> Result<SvgPoint, BoxError> {
        let a = self.next().ok_or("x coord")?.parse::<f32>()?;
        let b = self.next().ok_or("y coord")?.parse::<f32>()?;
        Ok(SvgPoint(a, b))
    }

    fn parse_cubic_curve_to(&mut self) -> Result<SvgCubicBezierCurveTo, BoxError> {
        let ctrl1 = self.parse_point()?;
        let ctrl2 = self.parse_point()?;
        let to = self.parse_point()?;
        Ok(SvgCubicBezierCurveTo { ctrl1, ctrl2, to })
    }

    fn parse_quadratic_curve_to(&mut self) -> Result<SvgQuadraticBezierCurveTo, BoxError> {
        let ctrl = self.parse_point()?;
        let to = self.parse_point()?;
        Ok(SvgQuadraticBezierCurveTo { ctrl, to })
    }
}

fn parse_svg_instructions(path: &str) -> Result<Vec<Instruction>, BoxError> {
    let mut instructions = Vec::new();
    let mut tokens = path.split(&[' ', '\n', '\r']).map(|s| s.trim()).filter(|s| s.len() > 0).into_iter();
    while let Some(token) = tokens.next() {
        instructions.push(match token {
            "M" => Instruction::MoveTo(tokens.parse_point()?),
            "L" => Instruction::LineTo(tokens.parse_point()?),
            "C" => Instruction::CubicCurveTo(tokens.parse_cubic_curve_to()?),
            "Q" => Instruction::QuadraticCurveTo(tokens.parse_quadratic_curve_to()?),
            "Z" => Instruction::ClosePath,
            ins => return Err(format!("Unknown instruction: {ins}").into())
        });
    }
    Ok(instructions)
}

pub fn simple_svg_to_path(path: &str) -> Path {
    let mut p = PathBuilder::new();
    for instruction in parse_svg_instructions(path).unwrap() {
        match instruction {
            Instruction::MoveTo(to) => {
                p.move_to(to.to_vec2());
            },
            Instruction::LineTo(to) => {
                p.line_to(to.to_vec2());
            },
            Instruction::CubicCurveTo(c) => {
                p.cubic_bezier_to(c.ctrl1.to_vec2(), c.ctrl2.to_vec2(), c.to.to_vec2());
            },
            Instruction::QuadraticCurveTo(c) => {
                p.quadratic_bezier_to(c.ctrl.to_vec2(), c.to.to_vec2());
            },
            Instruction::ClosePath => {
                p.close();
            },
        }
    }
    p.build()
}