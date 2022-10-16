//use colored::Colorize;
use rand::seq::SliceRandom;
use std::collections::HashMap;
// use std::env;
use std::ops::Index;
use std::ops::IndexMut;
use std::thread;
use std::{fs::File, io::Read};

use rand::thread_rng;

use crate::backgrounds::{
    // CHUNK_HEIGHT, CHUNK_WIDTH, 
    MAP_HEIGHT, MAP_WIDTH, 
    TILE_SIZE
};

pub(crate) fn wfc() -> Vec<Vec<usize>> {
    // let args: Vec<String> = env::args().collect();

    // println!("{args:?}");

    //let mut file = File::open("chars.txt").unwrap();
    let mut file = File::open("assets/backgrounds/input.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    //adjacency rules
    let mut tile_types: Vec<usize> = Vec::new();
    let mut rules: HashMap<usize, HashMap<Dir, Vec<usize>>> = HashMap::new();

    //input board (should only be characters)
    // let in_board: Vec<Vec<usize>> = contents.lines().map(|l| l.chars().collect()).collect();
    let in_board: Vec<Vec<usize>> = contents.lines()
    .map(|l| l.split(" ")
    .map(|s| s.parse::<usize>().unwrap())
    .collect::<Vec<usize>>())
    .collect::<Vec<Vec<usize>>>();
    // println!("{:?}", &in_board);




    //the real board
    //let mut board = board::new((100, 100), rules);

    //iterate over our input board row-major to generate rules
    for (row, line) in in_board.iter().enumerate() {
        for (col, c) in line.iter().enumerate() {
            //print!("{c}");

            if !tile_types.contains(&(*c as usize)) {
                tile_types.push(*c as usize);
            }

            //get the rules entry for this kind of tile
            //if it doesnt exist, add a tile_type vec for each direction
            let cur = rules.entry(*c as usize).or_insert(HashMap::from([
                (Dir::WEST, Vec::new()),
                (Dir::NORTH, Vec::new()),
                (Dir::EAST, Vec::new()),
                (Dir::SOUTH, Vec::new()),
            ]));

            //north
            row.checked_sub(1)
                .and_then(|r| in_board.get(r))
                .and_then(|c| c.get(col))
                .and_then(|e| {
                    let north_type = *e as usize;
                    cur.entry(Dir::NORTH).and_modify(|allowed| {
                        if !allowed.contains(&north_type) {
                            allowed.push(north_type);
                        }
                    });

                    Some(true)
                });

            //SOUTH
            row.checked_add(1)
                .and_then(|r| in_board.get(r))
                .and_then(|c| c.get(col))
                .and_then(|e| {
                    let north_type = *e as usize;
                    cur.entry(Dir::SOUTH).and_modify(|allowed| {
                        if !allowed.contains(&north_type) {
                            allowed.push(north_type);
                        }
                    });

                    Some(true)
                });

            //WEST

            col.checked_sub(1)
                .and_then(|col| in_board[row].get(col))
                .and_then(|char| {
                    let north_type = *char as usize;
                    cur.entry(Dir::WEST).and_modify(|allowed| {
                        if !allowed.contains(&north_type) {
                            allowed.push(north_type);
                        }
                    });

                    Some(true)
                });

            //EAST

            col.checked_add(1)
                .and_then(|col| in_board[row].get(col))
                .and_then(|char| {
                    let north_type = *char as usize;
                    cur.entry(Dir::EAST).and_modify(|allowed| {
                        if !allowed.contains(&north_type) {
                            allowed.push(north_type);
                        }
                    });

                    Some(true)
                });
        }
        //print!("\n");
    }

    println!("done rulegen");
    println!("{:?}", rules);

    let mut board = Board::new(
        (
            // CHUNK_HEIGHT,
            // CHUNK_WIDTH,
            MAP_HEIGHT,
            MAP_WIDTH,
        ),
        rules,
        tile_types,
    );

    /*for (k, v) in &board.rules {
        println!("{:?}: {:?}", k, v);
    }*/
    let mut result_map: Vec<Vec<usize>> = Vec::new();

    let builder = thread::Builder::new().stack_size(4194304);

    let handler = builder
        .spawn(move || {
            let solvable = board.collapse(board.chose_tile_to_collapse());
            println!("solved? {solvable:?}");

            println!("\n");
            for row in &board.map{
                let mut result_row: Vec<usize> = Vec::new();
                for c in row {
                    // print!("{}", char::from_u32(c.t.unwrap() as u32).unwrap());
                    print!("{}", c.t.unwrap());
                    result_row.push(c.t.unwrap());
                }
                result_map.push(result_row);
                print!("\n")
            }
            result_map
        })
        .unwrap();

    handler.join().unwrap()

    //println!("{:?}", board.map);
}

//TODO: reconsider internalizing the map within this struct.
//because it is a local variable, we cant pass around references to tiles within it without the borrow checker getting mad
//so we instead have to refer to tiles by coords: (usize,usize)
#[derive(Debug, Clone)]
struct Board {
    map: Vec<Vec<Tile>>,
    rules: HashMap<usize, HashMap<Dir, Vec<usize>>>,
    //tile_types: Vec<usize>,
}

impl Index<(usize, usize)> for Board {
    type Output = Tile;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.map[index.0][index.1]
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.map[index.0][index.1]
    }
}

impl Board {
    fn new(
        size: (usize, usize),
        rules: HashMap<usize, HashMap<Dir, Vec<usize>>>,
        tile_types: Vec<usize>,
    ) -> Self {
        let mut map: Vec<Vec<Tile>> = Vec::new();

        for row in 0..size.0 {
            map.push(Vec::new());
            for col in 0..size.1 {
                map[row].push(Tile::fresh((row, col), tile_types.clone()));
            }
        }

        Self {
            map,
            rules,
            //tile_types,
        }
    }

    //TODO: this has a fucking abysmal runtime, please figure out a way to make it better
    //this will make sure no tiles on the board are breaking adjacency rules. it will NOT check if we have a completed board
    fn valid_position(&self) -> bool {
        for row in &self.map {
            for col in row {
                //empty superpositions are not valid unless the tile has a concrete type
                if col.position.len() == 0 && col.t.is_none() {
                    return false;
                }

                //only way we could be breaking adjacency rules is if this tile has a concrete position and one of its neighbors
                //ALSO has a conrete position, which is not allowed beside it
                if col.t.is_some() {
                    for n in self.get_neighbors(col.coords) {
                        if n.tile.t.is_some() {
                            if !self.rules[&n.tile.t.unwrap()][&n.anti_direction]
                                .contains(&col.t.unwrap())
                            {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        return true;
    }

    fn is_solved(&self) -> bool {
        if !self.valid_position() {
            return false;
        }

        return !self.map.iter().flatten().any(|t| t.t == None);
    }

    //chose the tile on the board with the lowest entropy and return its coords within the map
    fn chose_tile_to_collapse(&self) -> (usize, usize) {
        return self
            .map
            .iter()
            .flatten()
            .min_by(|x, y| x.entropy().cmp(&y.entropy()))
            .unwrap()
            .coords;
    }

    //returns a neighbors struct, with inidces into the map of the neighboring tiles
    fn get_neighbors(&self, pos: (usize, usize)) -> Neighbors {
        let mut n = Neighbors::new();

        //north
        n.north = pos
            .0
            .checked_sub(1)
            .and_then(|e| self.map.get(e))
            .and_then(|f| Some(f[pos.1].clone()));

        //south
        n.south = pos
            .0
            .checked_add(1)
            .and_then(|e| self.map.get(e))
            .and_then(|f| Some(f[pos.1].clone()));

        //west
        n.west = pos
            .1
            .checked_sub(1)
            .and_then(|e| self.map[pos.0].get(e))
            .and_then(|f| Some(f.clone()));

        //east
        n.east = pos
            .1
            .checked_add(1)
            .and_then(|e| self.map[pos.0].get(e))
            .and_then(|f| Some(f.clone()));

        //println!("east: {:?}", n.east);

        //println!("neighbors: {:?}", n);

        return n;
    }

    //takes 1 tile, collapses its state down to a concrete type, and udpates its neighbors super-positions
    //returns a result of a tile vec, either to put it in the backup queue or immeadiately undo it
    //fn collapse(&mut self, center_tile: (usize, usize)) -> Result<Vec<tile>, Vec<tile>> {
    fn collapse(&mut self, center_tile: (usize, usize)) -> bool {
        //gtfo of here and return our way up the call stack if we have solved our board
        if self.is_solved() {
            return true;
        }

        //backup the superposition and zero it out
        let mut random_pos = self[center_tile].position.clone();
        let backup_pos = self[center_tile].position.clone();
        self[center_tile].position = Vec::new();

        //iterate through the possible positions of the superposition
        random_pos.shuffle(&mut thread_rng());
        for pos in random_pos {
            //tentatively give our tile this concrete position and give it a char rep
            self[center_tile].t = Some(pos);
            //self[center_tile].rep = tile_type_to_char(pos);

            //backup neighbors and
            // update neighbors superpositions according to the subposition we are trying
            let old_neighbors = self.get_neighbors(center_tile);
            for mut n in self.get_neighbors(center_tile) {
                n.tile
                    .position
                    .retain(|t| self.rules[&pos][&n.direction].contains(t));
            }

            //if this subposition is a valid position, call solve on the next tile to be collapse
            if self.valid_position() {
                // if we are not in a solved board, continue recursing, otherwise, return our way up the call stack
                if self.collapse(self.chose_tile_to_collapse()) {
                    return true;
                } else {
                }
            } else {
                for n in old_neighbors {
                    self[n.tile.coords] = n.tile.clone();
                }
            }
        }
        self[center_tile].t = None;
        self[center_tile].position = backup_pos.clone();
        return false;
    }
}

/*
#[derive(Debug)]
struct Neighbors {
    north: Option<(usize, usize)>,
    south: Option<(usize, usize)>,
    east: Option<(usize, usize)>,
    west: Option<(usize, usize)>,
}*/
#[derive(Debug)]
struct Neighbors {
    north: Option<Tile>,
    south: Option<Tile>,
    east: Option<Tile>,
    west: Option<Tile>,
}

struct NeighborIterElement {
    direction: Dir,
    anti_direction: Dir,
    tile: Tile,
}

impl IntoIterator for Neighbors {
    type Item = NeighborIterElement;
    type IntoIter = std::vec::IntoIter<NeighborIterElement>;

    fn into_iter(self) -> Self::IntoIter {
        let mut neighbors: Vec<NeighborIterElement> = Vec::new();

        self.north.and_then(|f| {
            Some(neighbors.push(NeighborIterElement {
                direction: Dir::NORTH,
                anti_direction: Dir::SOUTH,
                tile: f,
            }))
        });

        self.south.and_then(|f| {
            Some(neighbors.push(NeighborIterElement {
                direction: Dir::SOUTH,
                anti_direction: Dir::NORTH,
                tile: f,
            }))
        });

        self.east.and_then(|f| {
            Some(neighbors.push(NeighborIterElement {
                direction: Dir::EAST,
                anti_direction: Dir::WEST,
                tile: f,
            }))
        });

        self.west.and_then(|f| {
            Some(neighbors.push(NeighborIterElement {
                direction: Dir::WEST,
                anti_direction: Dir::EAST,
                tile: f,
            }))
        });

        return neighbors.into_iter();
    }
}

impl Neighbors {
    fn new() -> Self {
        Self {
            north: None,
            south: None,
            east: None,
            west: None,
        }
    }
}

#[derive(PartialEq, Hash, Eq, Debug, Clone)]
enum Dir {
    WEST,
    NORTH,
    EAST,
    SOUTH,
}

impl IntoIterator for Dir {
    type Item = Dir;
    type IntoIter = std::vec::IntoIter<Dir>;

    fn into_iter(self) -> Self::IntoIter {
        return vec![Dir::NORTH, Dir::SOUTH, Dir::EAST, Dir::WEST].into_iter();
    }
}

//this struct defines the rules for a tile type
#[derive(PartialEq, Hash, Eq, Debug, Clone)]
struct Tile {
    coords: (usize, usize),
    //rep: char,
    //tile only has a type once it has been fully collapsed
    t: Option<usize>,
    position: Vec<usize>,
}
impl Tile {
    fn fresh(coords: (usize, usize), full: Vec<usize>) -> Self {
        Self {
            coords,
            //rep: 'X',
            t: None,
            position: full,
        }
    }

    fn entropy(&self) -> usize {
        if self.t.is_some() {
            return usize::MAX;
        } else {
            return self.position.len();
        }
    }
}