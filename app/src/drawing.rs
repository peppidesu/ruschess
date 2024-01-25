use notan::prelude::*;
use notan::draw::*;
use ruschess_core::*;

use crate::state::*;

pub const COLOR_TILE_LIGHT: Color = Color { r: 0.835, g: 0.835, b: 0.835, a: 1.0 };
pub const COLOR_TILE_DARK: Color = Color { r: 0.247, g: 0.247, b: 0.247, a: 1.0 };
pub const COLOR_TILE_HOVER: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 };
pub const COLOR_TILE_SELECTED: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.4 };
pub const COLOR_TILE_MOVE: Color = Color { r: 1.0, g: 0.4, b: 0.08, a: 0.2 };

pub const BOARD_TILE_SIZE: f32 = 64.0;
pub const INFO_WIDTH: f32 = 256.0;
pub const SCREEN_WIDTH: f32 = BOARD_TILE_SIZE * 8.0 + INFO_WIDTH;
pub const SCREEN_HEIGHT: f32 = BOARD_TILE_SIZE * 8.0;


pub enum TileOverlayType {
    Hover,
    Selected,
    Move,
}

#[inline]
fn get_tile_color(x: usize, y: usize) -> Color {
    if (x + y) % 2 == 0 {
        COLOR_TILE_LIGHT
    } else {
        COLOR_TILE_DARK
    }
}

#[inline]
pub fn board_to_screen(perspective: PieceColor, position: Position) -> (f32, f32) {
    let (rank, file) = position.into();
    let (x, y) = (file as f32, rank as f32);

    match perspective {
        PieceColor::White => (x * BOARD_TILE_SIZE, (7.0 - y) * BOARD_TILE_SIZE),
        PieceColor::Black => ((7.0 - x) * BOARD_TILE_SIZE, y * BOARD_TILE_SIZE),
    }
}

#[inline]
pub fn screen_to_board(perspective: PieceColor, x: f32, y: f32) -> Option<Position> {
    let x = (x / BOARD_TILE_SIZE) as i8;
    let y = (y / BOARD_TILE_SIZE) as i8;

    if x < 0 || x > 7 || y < 0 || y > 7 {
        None
    } else {
        match perspective {
            PieceColor::White => Some(Position::new(7 - y as u8, x as u8)),
            PieceColor::Black => Some(Position::new(y as u8, 7 - x as u8)),
        }
    }
}

#[inline]
pub fn screen_to_board_unsafe(perspective: PieceColor, x: f32, y: f32) -> Position {
    let x = (x / BOARD_TILE_SIZE) as u8;
    let y = (y / BOARD_TILE_SIZE) as u8;

    match perspective {
        PieceColor::White => Position::new(7 - y, x),
        PieceColor::Black => Position::new(y, 7 - x),
    }
}

#[inline]
fn draw_board_tile(draw: &mut Draw, x: usize, y: usize) {
    draw.rect((x as f32, y as f32), (BOARD_TILE_SIZE, BOARD_TILE_SIZE))
        .color(get_tile_color(x, y));
}

#[inline]
fn draw_board_tile_overlay(draw: &mut Draw, x: usize, y: usize, overlay: TileOverlayType) {
    
    let color = match overlay {
        TileOverlayType::Hover => COLOR_TILE_HOVER,
        TileOverlayType::Selected => COLOR_TILE_SELECTED,
        TileOverlayType::Move => COLOR_TILE_MOVE,
    };

    draw.rect((x as f32, y as f32), (BOARD_TILE_SIZE, BOARD_TILE_SIZE))
        .color(color);
}

#[inline]
fn draw_piece(draw: &mut Draw, piece: Piece, position: Posit) {

}

fn draw_board(gfx: &mut Graphics, context: &mut DrawContext, board: &Board, perspective: PieceColor) {
    let mut draw = gfx.create_draw();
    for x in 0..8 {
        for y in 0..8 {
            draw_board_tile(&mut draw, x, y);
        }
    }


}













