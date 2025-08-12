use super::*;
use embedded_graphics::{mock_display::MockDisplay, pixelcolor::BinaryColor, primitives::Triangle};

fn triangle() -> impl PointsIter {
    let tri = Triangle::new(Point::new(0, 10), Point::new(0, 0), Point::new(20, 0));

    tri
}

#[test]
fn rot0() {
    let mut disp = Rotate0::new(MockDisplay::new());

    assert_eq!(
        disp.bounding_box(),
        Rectangle {
            top_left: Point::default(),
            size: Size {
                width: 64,
                height: 64
            }
        }
    );

    disp.draw_iter(triangle().points().map(|pt| Pixel(pt, BinaryColor::On)))
        .expect("draw failed");

    assert_eq!(
        disp.as_ref().affected_area(),
        Rectangle {
            top_left: Point { x: 0, y: 0 },
            size: Size {
                width: 21,
                height: 11
            }
        }
    );

    disp.as_ref().assert_pattern(&[
        "#####################",
        "###################  ",
        "#################    ",
        "###############      ",
        "#############        ",
        "###########          ",
        "#########            ",
        "#######              ",
        "#####                ",
        "###                  ",
        "#                    ",
    ]);
}

#[test]
fn flipy() {
    let mut disp = FlipY::new(MockDisplay::new());

    assert_eq!(
        disp.bounding_box(),
        Rectangle {
            top_left: Point::default(),
            size: Size {
                width: 64,
                height: 64
            }
        }
    );

    disp.draw_iter(triangle().points().map(|pt| Pixel(pt, BinaryColor::On)))
        .expect("draw failed");
    assert_eq!(
        disp.as_ref().affected_area(),
        Rectangle {
            top_left: Point { x: 0, y: 53 },
            size: Size {
                width: 21,
                height: 11
            }
        }
    );

    disp.as_ref().assert_pattern(&[
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "#                    ",
        "###                  ",
        "#####                ",
        "#######              ",
        "#########            ",
        "###########          ",
        "#############        ",
        "###############      ",
        "#################    ",
        "###################  ",
        "#####################",
    ]);
}

#[test]
fn rot90() {
    let mut disp = Rotate90::new(MockDisplay::new());

    disp.draw_iter(triangle().points().map(|pt| Pixel(pt, BinaryColor::On)))
        .expect("draw failed");

    assert_eq!(
        disp.as_ref().affected_area(),
        Rectangle {
            top_left: Point { x: 53, y: 0 },
            size: Size {
                width: 11,
                height: 21
            }
        }
    );

    disp.as_ref().assert_pattern(&[
        "                                                     ###########",
        "                                                      ##########",
        "                                                      ##########",
        "                                                       #########",
        "                                                       #########",
        "                                                        ########",
        "                                                        ########",
        "                                                         #######",
        "                                                         #######",
        "                                                          ######",
        "                                                          ######",
        "                                                           #####",
        "                                                           #####",
        "                                                            ####",
        "                                                            ####",
        "                                                             ###",
        "                                                             ###",
        "                                                              ##",
        "                                                              ##",
        "                                                               #",
        "                                                               #",
    ]);
}

#[test]
fn rot270() {
    let mut disp = Rotate270::new(MockDisplay::new());

    disp.draw_iter(triangle().points().map(|pt| Pixel(pt, BinaryColor::On)))
        .expect("draw failed");

    assert_eq!(
        disp.as_ref().affected_area(),
        Rectangle {
            top_left: Point { x: 0, y: 43 },
            size: Size {
                width: 11,
                height: 21
            }
        }
    );

    disp.as_ref().assert_pattern(&[
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "                     ",
        "#                    ",
        "#                    ",
        "##                   ",
        "##                   ",
        "###                  ",
        "###                  ",
        "####                 ",
        "####                 ",
        "#####                ",
        "#####                ",
        "######               ",
        "######               ",
        "#######              ",
        "#######              ",
        "########             ",
        "########             ",
        "#########            ",
        "#########            ",
        "##########           ",
        "##########           ",
        "###########          ",
    ]);
}

#[test]
fn solid_flipx() {
    let mut disp = FlipX::new(MockDisplay::new());

    disp.fill_solid(
        &Rectangle {
            top_left: Point::new(1, 1),
            size: Size {
                width: 5,
                height: 10,
            },
        },
        BinaryColor::On,
    )
    .expect("fill failed");

    assert_eq!(
        disp.as_ref().affected_area(),
        Rectangle {
            top_left: Point::new(64 - 1 - 1 - 5, 1),
            size: Size {
                width: 5,
                height: 10
            }
        }
    );
}

#[test]
fn solid_flipy() {
    let mut disp = FlipY::new(MockDisplay::new());

    disp.fill_solid(
        &Rectangle {
            top_left: Point::new(1, 1),
            size: Size {
                width: 5,
                height: 10,
            },
        },
        BinaryColor::On,
    )
    .expect("fill failed");

    assert_eq!(
        disp.as_ref().affected_area(),
        Rectangle {
            top_left: Point::new(1, 64 - 1 - 1 - 10),
            size: Size {
                width: 5,
                height: 10
            }
        }
    );
}

#[test]
fn solid_rot90() {
    let mut disp = Rotate90::new(MockDisplay::new());

    disp.fill_solid(
        &Rectangle {
            top_left: Point::new(1, 1),
            size: Size {
                width: 5,
                height: 10,
            },
        },
        BinaryColor::On,
    )
    .expect("fill failed");

    assert_eq!(
        disp.as_ref().affected_area(),
        Rectangle {
            top_left: Point::new(64 - 1 - 1 - 10, 1),
            size: Size {
                width: 10,
                height: 5
            }
        }
    );
}

#[test]
fn solid_rot180() {
    let mut disp = Rotate180::new(MockDisplay::new());

    disp.fill_solid(
        &Rectangle {
            top_left: Point::new(1, 1),
            size: Size {
                width: 5,
                height: 10,
            },
        },
        BinaryColor::On,
    )
    .expect("fill failed");

    assert_eq!(
        disp.as_ref().affected_area(),
        Rectangle {
            top_left: Point::new(64 - 1 - 1 - 5, 64 - 1 - 1 - 10),
            size: Size {
                width: 5,
                height: 10
            }
        }
    );
}

#[test]
fn solid_rot270() {
    let mut disp = Rotate270::new(MockDisplay::new());

    disp.fill_solid(
        &Rectangle {
            top_left: Point::new(1, 1),
            size: Size {
                width: 5,
                height: 10,
            },
        },
        BinaryColor::On,
    )
    .expect("fill failed");

    assert_eq!(
        disp.as_ref().affected_area(),
        Rectangle {
            top_left: Point::new(1, 64 - 1 - 1 - 5),
            size: Size {
                width: 10,
                height: 5
            }
        }
    );
}

#[test]
fn rect_transpose() {
    let rect = Rectangle {
        top_left: Point::new(20, 10),
        size: Size::new(5, 100),
    };

    fn lower_right(rect: &Rectangle) -> Point {
        Point::new(
            rect.top_left.x + rect.size.width as i32,
            rect.top_left.y + rect.size.height as i32,
        )
    }

    assert_eq!(lower_right(&rect), Point::new(25, 110));

    use crate::r#impl::Transpose;

    let rx = rect.transpose();

    assert_eq!(lower_right(&rx), Point::new(110, 25));
}
