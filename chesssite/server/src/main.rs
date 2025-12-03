use automerge::transaction::Transactable;
use automerge::ReadDoc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

const BROADCAST_CAPACITY: usize = 100;
const SESSION_TOKEN_LENGTH: usize = 32;

// Store for match broadcast channels - one per match for real-time updates
type MatchChannels = Arc<RwLock<HashMap<String, broadcast::Sender<MatchUpdate>>>>;

// Sessions store - maps session token to username
type Sessions = Arc<RwLock<HashMap<String, String>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatchUpdate {
    document: String,
    sender_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatchInfo {
    id: String,
    white_player: String,
    black_player: String,
    status: String,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterResponse {
    username: String,
    #[serde(rename = "createdAt")]
    created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    username: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MeResponse {
    username: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateMatchRequest {
    opponent_username: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateMatchResponse {
    id: String,
    white_player: String,
    black_player: String,
    status: String,
    #[serde(rename = "createdAt")]
    created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MatchDetailResponse {
    id: String,
    white_player: String,
    black_player: String,
    status: String,
    document: String,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListMatchesResponse {
    matches: Vec<MatchInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MoveRequest {
    from: String,
    to: String,
    promotion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MoveResponse {
    success: bool,
    document: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListUsersResponse {
    users: Vec<String>,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WsMessage {
    #[serde(rename = "identify")]
    Identify { client_id: String },
    #[serde(rename = "update")]
    Update { document: String },
    #[serde(rename = "sync")]
    Sync { document: String, sender_id: String },
    #[serde(rename = "connected")]
    Connected { document: String },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    channels: MatchChannels,
    sessions: Sessions,
}

// Chess pieces and board representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Piece {
    piece_type: PieceType,
    color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChessState {
    board: [[Option<Piece>; 8]; 8],
    current_turn: Color,
    moves: Vec<ChessMove>,
    status: String,
    white_can_castle_kingside: bool,
    white_can_castle_queenside: bool,
    black_can_castle_kingside: bool,
    black_can_castle_queenside: bool,
    en_passant_target: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChessMove {
    from: String,
    to: String,
    piece: Piece,
    captured: Option<Piece>,
    promotion: Option<PieceType>,
    timestamp: i64,
}

impl ChessState {
    fn new() -> Self {
        let mut board: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];

        // Set up white pieces
        board[0][0] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::White,
        });
        board[0][1] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::White,
        });
        board[0][2] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::White,
        });
        board[0][3] = Some(Piece {
            piece_type: PieceType::Queen,
            color: Color::White,
        });
        board[0][4] = Some(Piece {
            piece_type: PieceType::King,
            color: Color::White,
        });
        board[0][5] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::White,
        });
        board[0][6] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::White,
        });
        board[0][7] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::White,
        });
        for col in 0..8 {
            board[1][col] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::White,
            });
        }

        // Set up black pieces
        board[7][0] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::Black,
        });
        board[7][1] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::Black,
        });
        board[7][2] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::Black,
        });
        board[7][3] = Some(Piece {
            piece_type: PieceType::Queen,
            color: Color::Black,
        });
        board[7][4] = Some(Piece {
            piece_type: PieceType::King,
            color: Color::Black,
        });
        board[7][5] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::Black,
        });
        board[7][6] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::Black,
        });
        board[7][7] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::Black,
        });
        for col in 0..8 {
            board[6][col] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::Black,
            });
        }

        ChessState {
            board,
            current_turn: Color::White,
            moves: Vec::new(),
            status: "active".to_string(),
            white_can_castle_kingside: true,
            white_can_castle_queenside: true,
            black_can_castle_kingside: true,
            black_can_castle_queenside: true,
            en_passant_target: None,
        }
    }

    fn parse_square(square: &str) -> Option<(usize, usize)> {
        if square.len() != 2 {
            return None;
        }
        let chars: Vec<char> = square.chars().collect();
        let col = match chars[0] {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return None,
        };
        let row = match chars[1] {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => return None,
        };
        Some((row, col))
    }

    fn is_valid_move(&self, from: (usize, usize), to: (usize, usize), player_color: Color) -> bool {
        let piece = match self.board[from.0][from.1] {
            Some(p) => p,
            None => return false,
        };

        // Check if it's this player's piece
        if piece.color != player_color {
            return false;
        }

        // Check if it's this player's turn
        if piece.color != self.current_turn {
            return false;
        }

        // Can't capture own piece
        if let Some(target) = self.board[to.0][to.1] {
            if target.color == piece.color {
                return false;
            }
        }

        // Validate move based on piece type
        let valid_pattern = match piece.piece_type {
            PieceType::King => self.is_valid_king_move(from, to),
            PieceType::Queen => self.is_valid_queen_move(from, to),
            PieceType::Rook => self.is_valid_rook_move(from, to),
            PieceType::Bishop => self.is_valid_bishop_move(from, to),
            PieceType::Knight => self.is_valid_knight_move(from, to),
            PieceType::Pawn => self.is_valid_pawn_move(from, to, piece.color),
        };

        if !valid_pattern {
            return false;
        }

        // Check if move would leave king in check
        let mut test_state = self.clone();
        test_state.board[to.0][to.1] = test_state.board[from.0][from.1].take();
        if test_state.is_in_check(player_color) {
            return false;
        }

        true
    }

    fn is_valid_king_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let row_diff = (to.0 as i32 - from.0 as i32).abs();
        let col_diff = (to.1 as i32 - from.1 as i32).abs();

        // Normal king move
        if row_diff <= 1 && col_diff <= 1 {
            return true;
        }

        // Castling
        if row_diff == 0 && col_diff == 2 {
            let color = self.board[from.0][from.1]
                .map(|p| p.color)
                .unwrap_or(Color::White);
            if self.is_in_check(color) {
                return false;
            }

            // Kingside castling
            if to.1 == 6 {
                let can_castle = if color == Color::White {
                    self.white_can_castle_kingside
                } else {
                    self.black_can_castle_kingside
                };
                if can_castle && self.board[from.0][5].is_none() && self.board[from.0][6].is_none()
                {
                    // Check if king passes through check
                    let mut test_state = self.clone();
                    test_state.board[from.0][5] = test_state.board[from.0][from.1].take();
                    if !test_state.is_in_check(color) {
                        return true;
                    }
                }
            }

            // Queenside castling
            if to.1 == 2 {
                let can_castle = if color == Color::White {
                    self.white_can_castle_queenside
                } else {
                    self.black_can_castle_queenside
                };
                if can_castle
                    && self.board[from.0][1].is_none()
                    && self.board[from.0][2].is_none()
                    && self.board[from.0][3].is_none()
                {
                    // Check if king passes through check
                    let mut test_state = self.clone();
                    test_state.board[from.0][3] = test_state.board[from.0][from.1].take();
                    if !test_state.is_in_check(color) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn is_valid_queen_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        self.is_valid_rook_move(from, to) || self.is_valid_bishop_move(from, to)
    }

    fn is_valid_rook_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        if from.0 != to.0 && from.1 != to.1 {
            return false;
        }

        // Check path is clear
        if from.0 == to.0 {
            let (min_col, max_col) = if from.1 < to.1 {
                (from.1, to.1)
            } else {
                (to.1, from.1)
            };
            for col in (min_col + 1)..max_col {
                if self.board[from.0][col].is_some() {
                    return false;
                }
            }
        } else {
            let (min_row, max_row) = if from.0 < to.0 {
                (from.0, to.0)
            } else {
                (to.0, from.0)
            };
            for row in (min_row + 1)..max_row {
                if self.board[row][from.1].is_some() {
                    return false;
                }
            }
        }

        true
    }

    fn is_valid_bishop_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let row_diff = (to.0 as i32 - from.0 as i32).abs();
        let col_diff = (to.1 as i32 - from.1 as i32).abs();

        if row_diff != col_diff {
            return false;
        }

        // Check path is clear
        let row_dir = if to.0 > from.0 { 1i32 } else { -1 };
        let col_dir = if to.1 > from.1 { 1i32 } else { -1 };

        let mut row = from.0 as i32 + row_dir;
        let mut col = from.1 as i32 + col_dir;

        while row != to.0 as i32 && col != to.1 as i32 {
            if self.board[row as usize][col as usize].is_some() {
                return false;
            }
            row += row_dir;
            col += col_dir;
        }

        true
    }

    fn is_valid_knight_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let row_diff = (to.0 as i32 - from.0 as i32).abs();
        let col_diff = (to.1 as i32 - from.1 as i32).abs();

        (row_diff == 2 && col_diff == 1) || (row_diff == 1 && col_diff == 2)
    }

    fn is_valid_pawn_move(&self, from: (usize, usize), to: (usize, usize), color: Color) -> bool {
        let direction: i32 = if color == Color::White { 1 } else { -1 };
        let start_row = if color == Color::White { 1 } else { 6 };

        let row_diff = to.0 as i32 - from.0 as i32;
        let col_diff = (to.1 as i32 - from.1 as i32).abs();

        // Forward move
        if col_diff == 0 {
            if row_diff == direction && self.board[to.0][to.1].is_none() {
                return true;
            }
            // Double move from starting position
            if from.0 == start_row
                && row_diff == 2 * direction
                && self.board[to.0][to.1].is_none()
                && self.board[(from.0 as i32 + direction) as usize][from.1].is_none()
            {
                return true;
            }
        }

        // Capture
        if col_diff == 1 && row_diff == direction {
            // Normal capture
            if self.board[to.0][to.1].is_some() {
                return true;
            }
            // En passant
            if let Some(ep_target) = self.en_passant_target {
                if ep_target == (to.0, to.1) {
                    return true;
                }
            }
        }

        false
    }

    fn find_king(&self, color: Color) -> Option<(usize, usize)> {
        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.board[row][col] {
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return Some((row, col));
                    }
                }
            }
        }
        None
    }

    fn is_square_attacked(&self, square: (usize, usize), by_color: Color) -> bool {
        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.board[row][col] {
                    if piece.color == by_color {
                        let can_attack = match piece.piece_type {
                            PieceType::King => {
                                let row_diff = (square.0 as i32 - row as i32).abs();
                                let col_diff = (square.1 as i32 - col as i32).abs();
                                row_diff <= 1 && col_diff <= 1
                            }
                            PieceType::Queen => {
                                self.is_valid_rook_move((row, col), square)
                                    || self.is_valid_bishop_move((row, col), square)
                            }
                            PieceType::Rook => self.is_valid_rook_move((row, col), square),
                            PieceType::Bishop => self.is_valid_bishop_move((row, col), square),
                            PieceType::Knight => self.is_valid_knight_move((row, col), square),
                            PieceType::Pawn => {
                                let direction: i32 = if by_color == Color::White { 1 } else { -1 };
                                let row_diff = square.0 as i32 - row as i32;
                                let col_diff = (square.1 as i32 - col as i32).abs();
                                row_diff == direction && col_diff == 1
                            }
                        };
                        if can_attack {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_in_check(&self, color: Color) -> bool {
        if let Some(king_pos) = self.find_king(color) {
            let opponent_color = if color == Color::White {
                Color::Black
            } else {
                Color::White
            };
            return self.is_square_attacked(king_pos, opponent_color);
        }
        false
    }

    fn has_legal_moves(&self, color: Color) -> bool {
        for from_row in 0..8 {
            for from_col in 0..8 {
                if let Some(piece) = self.board[from_row][from_col] {
                    if piece.color == color {
                        for to_row in 0..8 {
                            for to_col in 0..8 {
                                if self.is_valid_move((from_row, from_col), (to_row, to_col), color)
                                {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn make_move(
        &mut self,
        from: (usize, usize),
        to: (usize, usize),
        promotion: Option<PieceType>,
    ) -> Option<ChessMove> {
        let piece = self.board[from.0][from.1]?;
        let captured = self.board[to.0][to.1];

        // Handle en passant capture
        let mut actual_captured = captured;
        if piece.piece_type == PieceType::Pawn {
            if let Some(ep_target) = self.en_passant_target {
                if ep_target == (to.0, to.1) {
                    let captured_row = if piece.color == Color::White {
                        to.0 - 1
                    } else {
                        to.0 + 1
                    };
                    actual_captured = self.board[captured_row][to.1].take();
                }
            }
        }

        // Update castling rights
        if piece.piece_type == PieceType::King {
            if piece.color == Color::White {
                self.white_can_castle_kingside = false;
                self.white_can_castle_queenside = false;
            } else {
                self.black_can_castle_kingside = false;
                self.black_can_castle_queenside = false;
            }

            // Handle castling - move the rook
            let col_diff = to.1 as i32 - from.1 as i32;
            if col_diff == 2 {
                // Kingside castling
                self.board[from.0][5] = self.board[from.0][7].take();
            } else if col_diff == -2 {
                // Queenside castling
                self.board[from.0][3] = self.board[from.0][0].take();
            }
        }

        if piece.piece_type == PieceType::Rook {
            if from == (0, 0) {
                self.white_can_castle_queenside = false;
            } else if from == (0, 7) {
                self.white_can_castle_kingside = false;
            } else if from == (7, 0) {
                self.black_can_castle_queenside = false;
            } else if from == (7, 7) {
                self.black_can_castle_kingside = false;
            }
        }

        // Set en passant target
        self.en_passant_target = None;
        if piece.piece_type == PieceType::Pawn {
            let row_diff = (to.0 as i32 - from.0 as i32).abs();
            if row_diff == 2 {
                let ep_row = if piece.color == Color::White {
                    from.0 + 1
                } else {
                    from.0 - 1
                };
                self.en_passant_target = Some((ep_row, from.1));
            }
        }

        // Move the piece
        self.board[from.0][from.1] = None;

        // Handle pawn promotion
        let final_piece = if piece.piece_type == PieceType::Pawn {
            let promotion_row = if piece.color == Color::White { 7 } else { 0 };
            if to.0 == promotion_row {
                let promo_type = promotion.unwrap_or(PieceType::Queen);
                Piece {
                    piece_type: promo_type,
                    color: piece.color,
                }
            } else {
                piece
            }
        } else {
            piece
        };

        self.board[to.0][to.1] = Some(final_piece);

        // Switch turns
        self.current_turn = if self.current_turn == Color::White {
            Color::Black
        } else {
            Color::White
        };

        // Check for checkmate or stalemate
        if !self.has_legal_moves(self.current_turn) {
            if self.is_in_check(self.current_turn) {
                let winner = if self.current_turn == Color::White {
                    "black"
                } else {
                    "white"
                };
                self.status = format!("checkmate:{}", winner);
            } else {
                self.status = "stalemate".to_string();
            }
        } else if self.is_in_check(self.current_turn) {
            self.status = "check".to_string();
        } else {
            self.status = "active".to_string();
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let chess_move = ChessMove {
            from: format!("{}{}", (b'a' + from.1 as u8) as char, from.0 + 1),
            to: format!("{}{}", (b'a' + to.1 as u8) as char, to.0 + 1),
            piece,
            captured: actual_captured,
            promotion,
            timestamp: now,
        };

        self.moves.push(chess_move.clone());

        Some(chess_move)
    }
}

fn generate_session_token() -> String {
    use aws_lc_rs::rand::{SecureRandom, SystemRandom};
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; SESSION_TOKEN_LENGTH];
    rng.fill(&mut bytes)
        .expect("Failed to generate random bytes");
    BASE64.encode(&bytes)
}

fn hash_password(password: &str, salt: &[u8]) -> Vec<u8> {
    use aws_lc_rs::pbkdf2;
    let mut out = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(100_000).expect("Non-zero value"),
        salt,
        password.as_bytes(),
        &mut out,
    );
    out.to_vec()
}

fn verify_password(password: &str, salt: &[u8], expected_hash: &[u8]) -> bool {
    use aws_lc_rs::pbkdf2;
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(100_000).expect("Non-zero value"),
        salt,
        password.as_bytes(),
        expected_hash,
    )
    .is_ok()
}

fn get_current_time() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            password_hash BLOB NOT NULL,
            salt BLOB NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS matches (
            id TEXT PRIMARY KEY,
            white_player TEXT NOT NULL,
            black_player TEXT NOT NULL,
            status TEXT NOT NULL,
            document BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (white_player) REFERENCES users(username),
            FOREIGN KEY (black_player) REFERENCES users(username)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_matches_players 
        ON matches(white_player, black_player)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn get_user_from_token(state: &AppState, token: &str) -> Option<String> {
    let sessions = state.sessions.read().await;
    sessions.get(token).cloned()
}

fn extract_token(auth_header: Option<&str>) -> Option<&str> {
    auth_header.and_then(|h| h.strip_prefix("Bearer "))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), StatusCode> {
    use aws_lc_rs::rand::{SecureRandom, SystemRandom};

    // Check if user already exists
    let existing: Option<(String,)> =
        sqlx::query_as("SELECT username FROM users WHERE username = ?")
            .bind(&payload.username)
            .fetch_optional(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to check existing user: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    if existing.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let rng = SystemRandom::new();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt).map_err(|err| {
        tracing::error!("Failed to generate salt: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let password_hash = hash_password(&payload.password, &salt);
    let now = get_current_time();

    sqlx::query(
        "INSERT INTO users (username, password_hash, salt, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .bind(salt.as_slice())
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to create user: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("User registered: {}", payload.username);

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            username: payload.username,
            created_at: now,
        }),
    ))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user: Option<(String, Vec<u8>, Vec<u8>)> =
        sqlx::query_as("SELECT username, password_hash, salt FROM users WHERE username = ?")
            .bind(&payload.username)
            .fetch_optional(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to get user: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let (username, password_hash, salt) = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_password(&payload.password, &salt, &password_hash) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = generate_session_token();

    {
        let mut sessions = state.sessions.write().await;
        sessions.insert(token.clone(), username.clone());
    }

    tracing::info!("User logged in: {}", username);

    Ok(Json(LoginResponse { username, token }))
}

async fn logout(State(state): State<AppState>, headers: axum::http::HeaderMap) -> StatusCode {
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = extract_token(Some(auth_str)) {
                let mut sessions = state.sessions.write().await;
                sessions.remove(token);
            }
        }
    }
    StatusCode::OK
}

async fn me(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<MeResponse>, StatusCode> {
    let auth_header = headers.get("Authorization").and_then(|v| v.to_str().ok());
    let token = extract_token(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;
    let username = get_user_from_token(&state, token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(Json(MeResponse { username }))
}

async fn list_users(State(state): State<AppState>) -> Result<Json<ListUsersResponse>, StatusCode> {
    let users: Vec<(String,)> = sqlx::query_as("SELECT username FROM users ORDER BY username ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to list users: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ListUsersResponse {
        users: users.into_iter().map(|(u,)| u).collect(),
    }))
}

fn create_initial_match_document(
    match_id: &str,
    white_player: &str,
    black_player: &str,
) -> Result<Vec<u8>, StatusCode> {
    let now = get_current_time();
    let chess_state = ChessState::new();
    let chess_state_json = serde_json::to_string(&chess_state).map_err(|err| {
        tracing::error!("Failed to serialize chess state: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut doc = automerge::AutoCommit::new();
    doc.put(automerge::ROOT, "id", match_id).map_err(|err| {
        tracing::error!("Failed to set match id: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    doc.put(automerge::ROOT, "whitePlayer", white_player)
        .map_err(|err| {
            tracing::error!("Failed to set white player: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    doc.put(automerge::ROOT, "blackPlayer", black_player)
        .map_err(|err| {
            tracing::error!("Failed to set black player: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    doc.put(automerge::ROOT, "status", "active")
        .map_err(|err| {
            tracing::error!("Failed to set status: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    doc.put(automerge::ROOT, "chessState", &chess_state_json)
        .map_err(|err| {
            tracing::error!("Failed to set chess state: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    doc.put(automerge::ROOT, "createdAt", now).map_err(|err| {
        tracing::error!("Failed to set createdAt: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    doc.put(automerge::ROOT, "updatedAt", now).map_err(|err| {
        tracing::error!("Failed to set updatedAt: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(doc.save())
}

async fn create_match(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateMatchRequest>,
) -> Result<(StatusCode, Json<CreateMatchResponse>), StatusCode> {
    let auth_header = headers.get("Authorization").and_then(|v| v.to_str().ok());
    let token = extract_token(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;
    let username = get_user_from_token(&state, token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify opponent exists
    let opponent: Option<(String,)> =
        sqlx::query_as("SELECT username FROM users WHERE username = ?")
            .bind(&payload.opponent_username)
            .fetch_optional(&state.db)
            .await
            .map_err(|err| {
                tracing::error!("Failed to get opponent: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    if opponent.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    if payload.opponent_username == username {
        return Err(StatusCode::BAD_REQUEST);
    }

    let match_id = uuid::Uuid::new_v4().to_string();
    let now = get_current_time();

    // Creator plays as white
    let white_player = username.clone();
    let black_player = payload.opponent_username.clone();

    let document = create_initial_match_document(&match_id, &white_player, &black_player)?;

    sqlx::query(
        "INSERT INTO matches (id, white_player, black_player, status, document, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&match_id)
    .bind(&white_player)
    .bind(&black_player)
    .bind("active")
    .bind(&document)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to create match: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        "Match created: {} (white: {}, black: {})",
        match_id,
        white_player,
        black_player
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateMatchResponse {
            id: match_id,
            white_player,
            black_player,
            status: "active".to_string(),
            created_at: now,
        }),
    ))
}

async fn list_matches(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ListMatchesResponse>, StatusCode> {
    let auth_header = headers.get("Authorization").and_then(|v| v.to_str().ok());
    let token = extract_token(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;
    let username = get_user_from_token(&state, token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let matches: Vec<(String, String, String, String, i64, i64)> = sqlx::query_as(
        "SELECT id, white_player, black_player, status, created_at, updated_at FROM matches WHERE white_player = ? OR black_player = ? ORDER BY updated_at DESC"
    )
    .bind(&username)
    .bind(&username)
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to list matches: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let match_list: Vec<MatchInfo> = matches
        .into_iter()
        .map(
            |(id, white_player, black_player, status, created_at, updated_at)| MatchInfo {
                id,
                white_player,
                black_player,
                status,
                created_at,
                updated_at,
            },
        )
        .collect();

    Ok(Json(ListMatchesResponse {
        matches: match_list,
    }))
}

async fn get_match(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<MatchDetailResponse>, StatusCode> {
    let row: Option<(String, String, String, String, Vec<u8>, i64, i64)> = sqlx::query_as(
        "SELECT id, white_player, black_player, status, document, created_at, updated_at FROM matches WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get match {}: {}", id, err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (id, white_player, black_player, status, document, created_at, updated_at) =
        row.ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(MatchDetailResponse {
        id,
        white_player,
        black_player,
        status,
        document: BASE64.encode(&document),
        created_at,
        updated_at,
    }))
}

async fn make_move(
    Path(id): Path<String>,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<MoveRequest>,
) -> Result<Json<MoveResponse>, StatusCode> {
    let auth_header = headers.get("Authorization").and_then(|v| v.to_str().ok());
    let token = extract_token(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;
    let username = get_user_from_token(&state, token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Get current match state
    let row: Option<(String, String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT white_player, black_player, status, document FROM matches WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get match {}: {}", id, err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (white_player, black_player, status, document_bytes) = row.ok_or(StatusCode::NOT_FOUND)?;

    // Check if game is still active
    if status != "active" && status != "check" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Determine player color
    let player_color = if username == white_player {
        Color::White
    } else if username == black_player {
        Color::Black
    } else {
        return Err(StatusCode::FORBIDDEN);
    };

    // Load automerge document
    let mut doc = automerge::AutoCommit::load(&document_bytes).map_err(|err| {
        tracing::error!("Failed to load document: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get chess state from document
    let chess_state_json = doc
        .get(automerge::ROOT, "chessState")
        .ok()
        .flatten()
        .and_then(|(v, _)| v.to_str().map(|s| s.to_string()))
        .ok_or_else(|| {
            tracing::error!("Missing chessState in document");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut chess_state: ChessState = serde_json::from_str(&chess_state_json).map_err(|err| {
        tracing::error!("Failed to parse chess state: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Parse move coordinates
    let from = ChessState::parse_square(&payload.from).ok_or(StatusCode::BAD_REQUEST)?;
    let to = ChessState::parse_square(&payload.to).ok_or(StatusCode::BAD_REQUEST)?;

    // Validate move
    if !chess_state.is_valid_move(from, to, player_color) {
        return Ok(Json(MoveResponse {
            success: false,
            document: BASE64.encode(&document_bytes),
        }));
    }

    // Parse promotion piece
    let promotion = payload.promotion.as_ref().map(|p| match p.as_str() {
        "queen" => PieceType::Queen,
        "rook" => PieceType::Rook,
        "bishop" => PieceType::Bishop,
        "knight" => PieceType::Knight,
        _ => PieceType::Queen,
    });

    // Make the move
    chess_state.make_move(from, to, promotion);

    // Update automerge document
    let new_chess_state_json = serde_json::to_string(&chess_state).map_err(|err| {
        tracing::error!("Failed to serialize chess state: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let now = get_current_time();
    doc.put(automerge::ROOT, "chessState", &new_chess_state_json)
        .map_err(|err| {
            tracing::error!("Failed to update chessState: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    doc.put(automerge::ROOT, "status", &chess_state.status)
        .map_err(|err| {
            tracing::error!("Failed to update status: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    doc.put(automerge::ROOT, "updatedAt", now).map_err(|err| {
        tracing::error!("Failed to update updatedAt: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let new_document = doc.save();

    // Determine match status for DB
    let db_status =
        if chess_state.status.starts_with("checkmate:") || chess_state.status == "stalemate" {
            chess_state.status.clone()
        } else {
            "active".to_string()
        };

    // Update database
    sqlx::query("UPDATE matches SET document = ?, status = ?, updated_at = ? WHERE id = ?")
        .bind(&new_document)
        .bind(&db_status)
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update match {}: {}", id, err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Broadcast update to WebSocket clients
    broadcast_update(&state, &id, &new_document, &username).await;

    Ok(Json(MoveResponse {
        success: true,
        document: BASE64.encode(&new_document),
    }))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, id, state))
}

async fn handle_socket(socket: WebSocket, match_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Get or create broadcast channel for this match
    let rx = {
        let mut channels = state.channels.write().await;
        let tx = channels
            .entry(match_id.clone())
            .or_insert_with(|| broadcast::channel(BROADCAST_CAPACITY).0);
        tx.subscribe()
    };

    let mut rx = rx;

    // Shared client ID
    let client_id: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

    // Send current match state on connection
    {
        let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT document FROM matches WHERE id = ?")
            .bind(&match_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

        if let Some((binary,)) = row {
            let msg = WsMessage::Connected {
                document: BASE64.encode(&binary),
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }
        }
    }

    // Spawn task to forward broadcast messages
    let client_id_for_broadcast = client_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            let should_send = {
                let client_id_guard = client_id_for_broadcast.read().await;
                client_id_guard
                    .as_ref()
                    .map_or(true, |id| id != &update.sender_id)
            };

            if should_send {
                let msg = WsMessage::Sync {
                    document: update.document,
                    sender_id: update.sender_id,
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Handle incoming messages
    let client_id_for_recv = client_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        if let WsMessage::Identify { client_id: cid } = ws_msg {
                            let mut client_id_guard = client_id_for_recv.write().await;
                            *client_id_guard = Some(cid);
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    tracing::info!("WebSocket connection closed for match {}", match_id);
}

async fn broadcast_update(state: &AppState, match_id: &str, binary: &[u8], sender_id: &str) {
    let channels = state.channels.read().await;
    if let Some(tx) = channels.get(match_id) {
        let update = MatchUpdate {
            document: BASE64.encode(binary),
            sender_id: sender_id.to_string(),
        };
        let _ = tx.send(update);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:chess.db?mode=rwc".to_string());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    init_db(&pool).await.expect("Failed to initialize database");

    let state = AppState {
        db: pool,
        channels: Arc::new(RwLock::new(HashMap::new())),
        sessions: Arc::new(RwLock::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Auth endpoints
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/me", get(me))
        // User endpoints
        .route("/api/users", get(list_users))
        // Match endpoints
        .route("/api/matches", get(list_matches).post(create_match))
        .route("/api/matches/{id}", get(get_match))
        .route("/api/matches/{id}/move", post(make_move))
        // WebSocket
        .route("/ws/matches/{id}", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "4001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    tracing::info!("Chess server listening on http://{}", addr);
    tracing::info!("WebSocket endpoint: ws://{}/ws/matches/:id", addr);
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_board_setup() {
        let state = ChessState::new();

        // Check white pieces
        assert_eq!(
            state.board[0][0].map(|p| p.piece_type),
            Some(PieceType::Rook)
        );
        assert_eq!(
            state.board[0][4].map(|p| p.piece_type),
            Some(PieceType::King)
        );
        assert_eq!(
            state.board[1][4].map(|p| p.piece_type),
            Some(PieceType::Pawn)
        );

        // Check black pieces
        assert_eq!(
            state.board[7][0].map(|p| p.piece_type),
            Some(PieceType::Rook)
        );
        assert_eq!(
            state.board[7][4].map(|p| p.piece_type),
            Some(PieceType::King)
        );
        assert_eq!(
            state.board[6][4].map(|p| p.piece_type),
            Some(PieceType::Pawn)
        );

        assert_eq!(state.current_turn, Color::White);
    }

    #[test]
    fn test_parse_square() {
        assert_eq!(ChessState::parse_square("a1"), Some((0, 0)));
        assert_eq!(ChessState::parse_square("e4"), Some((3, 4)));
        assert_eq!(ChessState::parse_square("h8"), Some((7, 7)));
        assert_eq!(ChessState::parse_square("invalid"), None);
    }

    #[test]
    fn test_pawn_move() {
        let state = ChessState::new();

        // White pawn e2 to e4
        assert!(state.is_valid_move((1, 4), (3, 4), Color::White));
        // White pawn e2 to e3
        assert!(state.is_valid_move((1, 4), (2, 4), Color::White));
        // Invalid: e2 to e5
        assert!(!state.is_valid_move((1, 4), (4, 4), Color::White));
    }

    #[test]
    fn test_knight_move() {
        let state = ChessState::new();

        // Knight b1 to c3
        assert!(state.is_valid_move((0, 1), (2, 2), Color::White));
        // Knight b1 to a3
        assert!(state.is_valid_move((0, 1), (2, 0), Color::White));
        // Invalid: Knight b1 to b3
        assert!(!state.is_valid_move((0, 1), (2, 1), Color::White));
    }

    #[test]
    fn test_make_move() {
        let mut state = ChessState::new();

        // Move e2 to e4
        let chess_move = state.make_move((1, 4), (3, 4), None);
        assert!(chess_move.is_some());
        assert_eq!(state.current_turn, Color::Black);
        assert!(state.board[1][4].is_none());
        assert!(state.board[3][4].is_some());
    }
}
