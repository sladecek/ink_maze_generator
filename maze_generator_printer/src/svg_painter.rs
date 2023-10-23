use crate::{
    shapes::{Mapper, Shapes, WallType},
};
use xmlwriter::{Options, XmlWriter};

const SVG_URL: &str = "http://www.w3.org/2000/svg";

fn paint_line(
    xml: &mut XmlWriter,
    mapper: &Mapper,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    style: &str,
) {
    xml.start_element("line");
    xml.write_attribute("xmlns", SVG_URL);
    xml.write_attribute("x1", &mapper.map_x(x1, y1));
    xml.write_attribute("y1", &mapper.map_y(x1, y1));
    xml.write_attribute("x2", &mapper.map_x(x2, y2));
    xml.write_attribute("y2", &mapper.map_y(x2, y2));
    xml.write_attribute("style", style);
    xml.end_element();
}

fn paint_circle(xml: &mut XmlWriter, mapper: &Mapper, cx: i32, cy: i32, radius: i32, style: &str) {
    xml.start_element("circle");
    xml.write_attribute("xmlns", SVG_URL);
    xml.write_attribute("cx", &mapper.map_x(cx, cy));
    xml.write_attribute("cy", &mapper.map_y(cx, cy));
    xml.write_attribute("r", &format!("{}", radius));
    xml.write_attribute("fill", "none");
    xml.write_attribute("style", style);
    xml.end_element();
}

fn paint_arc(
    xml: &mut XmlWriter,
    mapper: &Mapper,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    style: &str,
) {
    assert!(y1 == y2, "arc segment must be defined on the same diameter");

    xml.start_element("path");
    xml.write_attribute("xmlns", SVG_URL);

    let mx1 = &mapper.map_x(x1, y1);
    let my1 = &mapper.map_y(x1, y1);
    let mx2 = &mapper.map_x(x2, y2);
    let my2 = &mapper.map_y(x2, y2);

    let pth = format!("M{} {} A{} {} 0 0 1 {} {}", mx1, my1, y2, y2, mx2, my2);

    xml.write_attribute("d", &pth);
    xml.write_attribute("style", style);
    xml.write_attribute("fill", "none");
    xml.end_element();
}

fn paint_mark(xml: &mut XmlWriter, mapper: &Mapper, cx: i32, cy: i32, radius: i32, fill: &str) {
    xml.start_element("circle");
    xml.write_attribute("xmlns", SVG_URL);
    xml.write_attribute("cx", &mapper.map_x(cx, cy));
    xml.write_attribute("cy", &mapper.map_y(cx, cy));
    xml.write_attribute("r", &format!("{}", radius));
    xml.write_attribute("fill", fill);
    xml.end_element();
}

fn paint_walls(xml: &mut XmlWriter, shapes: &Shapes, instance: &Vec<bool>) {
    for wall in &shapes.walls {
        let style = match wall.t {
            WallType::Inner => "stroke:black;stroke-width:1",
            WallType::Outer => "stroke:black;stroke-width:3",
        };
        if (wall.wall < 0) || instance[wall.wall as usize] {
            if shapes.is_polar && wall.y1 == wall.y2 {
                // lines in polar coordinates are arcs
                if wall.x1 == wall.x2 {
                    // special case - full circle
                    paint_circle(xml, &shapes.mapper, 0, 0, wall.y1, style);
                } else {
                    paint_arc(
                        xml,
                        &shapes.mapper,
                        wall.x1,
                        wall.y1,
                        wall.x2,
                        wall.y2,
                        style,
                    )
                }
            } else {
                paint_line(
                    xml,
                    &shapes.mapper,
                    wall.x1,
                    wall.y1,
                    wall.x2,
                    wall.y2,
                    style,
                );
            }
        }
    }
}

pub fn paint_marks(xml: &mut XmlWriter, shapes: &Shapes, start_room: i32, target_room: i32) {
    for f in &shapes.floors {
        let mut radius = 0;
        let mut fill = "";
        if f.room == start_room {
            radius = 16;
            fill = "rgb(255,0,0)"
        } else if f.room == target_room {
            radius = 16;
            fill = "rgb(0,255,0)"
        }
        if radius > 0 {
            paint_mark(xml, &shapes.mapper, f.x, f.y, radius, fill);
        }
    }
}

pub fn paint_shapes(shapes: &Shapes, instance: &Vec<bool>, start_room: i32, target_room: i32) -> String {
    let mut xml = XmlWriter::new(Options::default());
    xml.start_element("svg");
    xml.write_attribute("xmlns", SVG_URL);
    xml.write_attribute_fmt(
        "viewBox",
        format_args!(
            "{} {} {} {}",
            0, 0, shapes.mapper.canvas_width, shapes.mapper.canvas_height
        ),
    );
    paint_walls(&mut xml, &shapes, instance);
    paint_marks(&mut xml, shapes, start_room, target_room);
    let result = xml.end_document();
    result
}
