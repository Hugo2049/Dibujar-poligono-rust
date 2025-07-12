use image::{ImageBuffer, Rgb, RgbImage};
use std::collections::HashMap;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[derive(Clone, Copy, Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

struct Framebuffer {
    buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
    width: u32,
    height: u32,
}

impl Framebuffer {
    fn new(width: u32, height: u32) -> Self {
        let buffer = ImageBuffer::new(width, height);
        Framebuffer {
            buffer,
            width,
            height,
        }
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.buffer.put_pixel(x as u32, y as u32, Rgb([color.r, color.g, color.b]));
        }
    }

    fn get_pixel(&self, x: i32, y: i32) -> Option<Color> {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let pixel = self.buffer.get_pixel(x as u32, y as u32);
            Some(Color::new(pixel[0], pixel[1], pixel[2]))
        } else {
            None
        }
    }

    fn save(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer.save(filename)?;
        Ok(())
    }
}

// Algoritmo de Bresenham para dibujar líneas
fn draw_line(fb: &mut Framebuffer, p1: Point, p2: Point, color: Color) {
    let mut x0 = p1.x;
    let mut y0 = p1.y;
    let x1 = p2.x;
    let y1 = p2.y;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        fb.set_pixel(x0, y0, color);

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}

// Algoritmo de relleno de polígonos usando scanline (relleno interno)
fn fill_polygon(fb: &mut Framebuffer, vertices: &[Point], fill_color: Color, line_color: Color) {
    if vertices.len() < 3 {
        return;
    }

    // Primero relleno, luego líneas para que las líneas queden encima
    
    // Encontrar límites del polígono
    let min_y = vertices.iter().map(|p| p.y).min().unwrap();
    let max_y = vertices.iter().map(|p| p.y).max().unwrap();

    // Para cada línea de escaneo
    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        // Encontrar intersecciones con las aristas del polígono
        for i in 0..vertices.len() {
            let next_i = (i + 1) % vertices.len();
            let p1 = vertices[i];
            let p2 = vertices[next_i];

            // Verificar si la línea de escaneo intersecta con esta arista
            // Mejorar la condición para evitar problemas con vértices horizontales
            if p1.y != p2.y && ((p1.y <= y && y < p2.y) || (p2.y <= y && y < p1.y)) {
                // Calcular la coordenada x de la intersección
                let x = p1.x + ((y - p1.y) * (p2.x - p1.x)) / (p2.y - p1.y);
                intersections.push(x);
            }
        }

        // Ordenar las intersecciones por coordenada x
        intersections.sort();

        // Rellenar entre pares de intersecciones (sin tocar los bordes)
        for chunk in intersections.chunks(2) {
            if chunk.len() == 2 {
                let x1 = chunk[0] + 1; // +1 para no tocar el borde izquierdo
                let x2 = chunk[1] - 1; // -1 para no tocar el borde derecho
                if x1 <= x2 {
                    for x in x1..=x2 {
                        fb.set_pixel(x, y, fill_color);
                    }
                }
            }
        }
    }

    // Ahora dibujamos las líneas del polígono encima del relleno
    for i in 0..vertices.len() {
        let next_i = (i + 1) % vertices.len();
        draw_line(fb, vertices[i], vertices[next_i], line_color);
    }
}

// Algoritmo de relleno con agujeros (relleno interno) - método simplificado
fn fill_polygon_with_holes(fb: &mut Framebuffer, outer_vertices: &[Point], holes: &[&[Point]], fill_color: Color, hole_color: Color, line_color: Color) {
    if outer_vertices.len() < 3 {
        return;
    }

    // Primero rellenar el polígono exterior completamente
    fill_polygon_interior(fb, outer_vertices, fill_color);

    // Luego rellenar los agujeros con su color específico
    for hole in holes {
        fill_polygon_interior(fb, hole, hole_color);
    }

    // Finalmente dibujar todas las líneas encima
    for i in 0..outer_vertices.len() {
        let next_i = (i + 1) % outer_vertices.len();
        draw_line(fb, outer_vertices[i], outer_vertices[next_i], line_color);
    }

    for hole in holes {
        for i in 0..hole.len() {
            let next_i = (i + 1) % hole.len();
            draw_line(fb, hole[i], hole[next_i], line_color);
        }
    }
}

// Función auxiliar para relleno interior sin dibujar líneas
fn fill_polygon_interior(fb: &mut Framebuffer, vertices: &[Point], fill_color: Color) {
    if vertices.len() < 3 {
        return;
    }

    let min_y = vertices.iter().map(|p| p.y).min().unwrap();
    let max_y = vertices.iter().map(|p| p.y).max().unwrap();

    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        for i in 0..vertices.len() {
            let next_i = (i + 1) % vertices.len();
            let p1 = vertices[i];
            let p2 = vertices[next_i];

            // Mejorar la condición para evitar problemas con vértices horizontales
            if p1.y != p2.y && ((p1.y <= y && y < p2.y) || (p2.y <= y && y < p1.y)) {
                let x = p1.x + ((y - p1.y) * (p2.x - p1.x)) / (p2.y - p1.y);
                intersections.push(x);
            }
        }

        intersections.sort();

        for chunk in intersections.chunks(2) {
            if chunk.len() == 2 {
                let x1 = chunk[0] + 1; // +1 para no tocar el borde izquierdo
                let x2 = chunk[1] - 1; // -1 para no tocar el borde derecho
                if x1 <= x2 {
                    for x in x1..=x2 {
                        fb.set_pixel(x, y, fill_color);
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fb = Framebuffer::new(WIDTH, HEIGHT);

    // Limpiar el framebuffer con color blanco
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            fb.set_pixel(x as i32, y as i32, Color::new(255, 255, 255));
        }
    }

    // Definir los polígonos
    let polygon1 = vec![
        Point::new(165, 380), Point::new(185, 360), Point::new(180, 330), Point::new(207, 345),
        Point::new(233, 330), Point::new(230, 360), Point::new(250, 380), Point::new(220, 385),
        Point::new(205, 410), Point::new(193, 383)
    ];

    let polygon2 = vec![
        Point::new(321, 335), Point::new(288, 286), Point::new(339, 251), Point::new(374, 302)
    ];

    let polygon3 = vec![
        Point::new(377, 249), Point::new(411, 197), Point::new(436, 249)
    ];

    let polygon4 = vec![
        Point::new(413, 177), Point::new(448, 159), Point::new(502, 88), Point::new(553, 53),
        Point::new(535, 36), Point::new(676, 37), Point::new(660, 52), Point::new(750, 145),
        Point::new(761, 179), Point::new(672, 192), Point::new(659, 214), Point::new(615, 214),
        Point::new(632, 230), Point::new(580, 230), Point::new(597, 215), Point::new(552, 214),
        Point::new(517, 144), Point::new(466, 180)
    ];

    // Agujero dentro del polígono 4
    let hole1 = vec![
        Point::new(682, 175), Point::new(708, 120), Point::new(735, 148), Point::new(739, 170)
    ];

    // Definir colores
    let red = Color::new(255, 0, 0);
    let green = Color::new(0, 255, 0);
    let blue = Color::new(0, 0, 255);
    let yellow = Color::new(255, 255, 0);
    let white = Color::new(255, 255, 255);
    let black = Color::new(0, 0, 0);

    // Rellenar los polígonos
    fill_polygon(&mut fb, &polygon1, red, black);
    fill_polygon(&mut fb, &polygon2, green, black);
    fill_polygon(&mut fb, &polygon3, blue, black);
    
    // Polígono 4 con agujero (polígono 5 se pinta de blanco)
    fill_polygon_with_holes(&mut fb, &polygon4, &[&hole1], yellow, white, black);

    // Guardar la imagen
    fb.save("out.png")?;
    println!("Imagen guardada como out.png");

    Ok(())
}
