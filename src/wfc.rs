// Authors: Chris Hinson, Dan Li

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

/// Generate a fixed (map) sized screen using wave function collapse
/// and return a 2D vector of indexes into the texture atlas.
pub(crate) fn wfc() -> Vec<Vec<usize>> {
    // let args: Vec<String> = env::args().collect();

    // println!("{args:?}");

    //let mut file = File::open("chars.txt").unwrap();
    let mut file = File::open("assets/backgrounds/input.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // Initialize 
    let mut tile_types: Vec<usize> = Vec::new();
    let mut rules: HashMap<usize, HashMap<Dir, Vec<usize>>> = HashMap::new();

    // Board input is a text file of usize indexes used to map to a tile index in
    // the texture atlas
    // Split file into lines
    let in_board: Vec<Vec<usize>> = contents.lines()
    // Split each line by spaces
    .map(|l| l.split(" ")
    // Parse as a usize
    .map(|s| s.parse::<usize>().unwrap())
    // Collect the split line into a single vector
    .collect::<Vec<usize>>())
    // Collect all the line vectors into a 2D vector
    .collect::<Vec<Vec<usize>>>();

    //the real board
    //let mut board = board::new((100, 100), rules);

    // Iterate over the board in row major order and generate rules
    for (row, line) in in_board.iter().enumerate() {
        for (col, tile_type) in line.iter().enumerate() {
            //print!("{c}");

            // If we've never seen this tile type before,
            // add it to our list of all seen tile types.
            if !tile_types.contains(&(*tile_type as usize)) {
                tile_types.push(*tile_type as usize);
            }

            // Get the adjacency rules for this tile type if they exist,
            // or insert a new adjacency rule map if none exist.
            // The adjacency matrix will be a hashmap mapping from neighbor
            // direction to a vector of allowed tiles for that direction.
            let cur = rules
                .entry(*tile_type as usize)
                .or_insert(
                    HashMap::from([
                        (Dir::WEST, Vec::new()),
                        (Dir::NORTH, Vec::new()),
                        (Dir::EAST, Vec::new()),
                        (Dir::SOUTH, Vec::new()),
                    ])
                );

            // Below we actually add the neighbors we see on this iteration to
            // the appropriate rule vector based on the direction of each neighbor
            // NORTH
            row.checked_sub(1)
                .and_then(|r| in_board.get(r))
                .and_then(|c| c.get(col))
                .and_then(|e| {
                    // Get type of northern neighbor
                    let north_type = *e as usize;
                    // Add this type to the allowed types 
                    // if it doesn't already exist there.
                    cur.entry(Dir::NORTH).and_modify(|allowed| {
                        if !allowed.contains(&north_type) {
                            allowed.push(north_type);
                        }
                    });


                    // Required by and_then
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

    // Check which rules we produced
    // println!("done rulegen");
    // println!("{:?}", rules);

    // Create the board with a specific height and width 
    // (HEIGHT COMES FIRST because ROW MAJOR order)
    // and the rules and tile types the board will use.
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
    // The result of our WFC operation, what we will want to return.
    let mut result_map: Vec<Vec<usize>> = Vec::new();

    // Let's spawn a new thread to do this with a large amount of stack memory
    // because of how many calls this might make.
    let builder = thread::Builder::new().stack_size(4194304);

    // Handler just is so we can join the thread back in
    let handler = builder
        .spawn(move || {
            // Collapse gives us a boolean telling us whether the 
            // board is in a collapsable state or if no valid option is available here.
            // This is the core part of WFC. Collapse will modify board to
            // be in a collapsed state, if possible.
            let solvable = board.collapse();
            println!("solved? {solvable:?}");
            println!("\n");

            // Put the board contents into the result
            // so we can return it.
            for row in &board.map {
                let mut result_row: Vec<usize> = Vec::new();
                for c in row {
                    // print!("{}", char::from_u32(c.t.unwrap() as u32).unwrap());
                    print!("{}", c.t.unwrap());
                    result_row.push(c.t.unwrap());
                }
                result_map.push(result_row);
                print!("\n")
            }

            // Return the map we created from the thread
            result_map
        })
        .unwrap();
    
    // Join thread into parent
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
    /// Initialize a board
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

    // TODO: Very poor runtime, needs optimization.
    /// This will make sure no tiles on the board are breaking adjacency rules. 
    /// It will NOT check if we have a completed board.
    fn valid_position(&self) -> bool {
        for row in &self.map {
            for col in row {
                // Empty superpositions are invalid unless the tile has a concrete type
                if col.position.len() == 0 && col.t.is_none() {
                    return false;
                }

                // Only way we could be breaking adjacency rules is if this tile has a concrete position and one of its neighbors
                // ALSO has a conrete position, which is not allowed beside it.
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

    /// Choose the tile on the board with the lowest entropy and return its coords within the map
    fn choose_tile_to_collapse(&self) -> (usize, usize) {
        return self
            .map
            .iter()
            .flatten()
            .min_by(|x, y| x.entropy().cmp(&y.entropy()))
            .unwrap()
            .coords;
    }

    /// Returns a neighbors struct, with inidces into the map of the neighboring tiles
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

    /// Takes 1 tile, collapses its state down to a concrete type, and udpates its neighbors' superpositions.
    /// Returns true on success or false on failure.
    fn collapse(&mut self) -> bool {

        let mut backup_stack: UndoStack = Vec::new();

        // Everything in here should become iterative
        while !self.is_solved() {

            // // If our board is already marked as solved, we're done here, 
            // // jump back up the stack.
            // Redundant
            if self.is_solved() {
                return true;
            }

            // This choose function picks the tile on the board with the least entropy
            // (that means the least number of possible states)
            let collapsing_tile = self.choose_tile_to_collapse();

            // center_tile is the tile we are pivoting collapse on right now.
            // Get the super position of the tile, back it up,
            // and empty out the position of this tile.
            let mut random_pos = self[collapsing_tile].position.clone();
            let backup_pos = self[collapsing_tile].position.clone();
            self[collapsing_tile].position = Vec::new();

            // Shuffle up the position and check all of them to find one that is valid.
            // This shuffle gives us the randomality of WFC
            random_pos.shuffle(&mut thread_rng());
            for pos in random_pos {
                // Tentatively give our tile this concrete position
                self[collapsing_tile].t = Some(pos);

                self.map.iter().map(
                    |row| 
                        row.iter().map(
                            |tile| 
                                println!("{:?}", tile)
                            )
                    );

                // Backup our neighbors and update neighbors' superpositions
                // according to the current subposition we are trying to collapse
                let old_neighbors = self.get_neighbors(collapsing_tile);

                // Make a backup
                backup_stack.push(UndoStackElement {
                    coords: collapsing_tile,
                    tile_kind: pos,
                    position: backup_pos.clone(),
                    neighbors: old_neighbors.clone(),
                });

                for mut n in self.get_neighbors(collapsing_tile) {
                    n.tile
                        .position
                        // Keep only subpositions that are valid in relation to our
                        // ruleset
                        .retain(|t| self.rules[&pos][&n.direction].contains(t));
                }

                // If this subposition is a valid position,
                // call solve on the next tile to be collapse
                if self.valid_position() {
                    // If we are not in a solved board, continue iterating, otherwise, return our way up the call stack
                    continue;
                } else {
                    // If we're not in a valid position,
                    // set ourselves back to what we were before
                    // trying and then we will try the next subpos.
                    for n in old_neighbors {
                        self[n.tile.coords] = n.tile.clone();
                    }
                }
            }
            // Reset ourselves and fail if no position 
            // ever succeeded. 
            // Should we maybe do this inside the for loop?
            // Don't think so, because then we give back our old position,
            // which includes the one we just tried.
            let resetting_tile = backup_stack.pop();
            match resetting_tile {
                Some(backup) => {
                    self[backup.coords].t = None;
                    self[backup.coords].position = backup.position.clone();
                    // for n in backup.neighbors {
                    //     self[n.tile.coords] = n.tile.clone();
                    // }
                },
                None => return false,
            }
            
            // self[collapsing_tile].t = None;
            // self[collapsing_tile].position = backup_pos.clone();
            // return false;
        } // end of while loop

        // If while loop broke then we succeeded
        return true;
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
#[derive(Debug, Clone)]
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
    // Tile only has a type once it has been fully collapsed
    t: Option<usize>,
    position: Vec<usize>,
}
impl Tile {
    /// Create a fresh tile with
    /// a full superposition. Subpositions will be removed 
    /// as collapse occurs.
    fn fresh(coords: (usize, usize), full: Vec<usize>) -> Self {
        Self {
            coords,
            //rep: 'X',
            t: None,
            position: full,
        }
    }

    /// Determine what the entropy of the tile is
    /// 
    /// Entropy is defined as the number of possible subpositions
    /// in this tile's superposition, or infinity if it is collapsed.
    fn entropy(&self) -> usize {
        if self.t.is_some() {
            return usize::MAX;
        } else {
            return self.position.len();
        }
    }
}

/// Bookkeeping struct
/// 
/// Contains the coordinates, chosen tile, old position, and old neighbor 
/// states for a tile, so that when we iterate we can undo operations that were done
/// in the order we did them.
struct UndoStackElement {
    // Coordinates of the tile we changed with that operation
    coords: (usize, usize), 
    // Type we chose for that tile
    tile_kind: usize, 
    // Old superposition
    position: Vec<usize>,
    // Old neighbors
    neighbors: Neighbors
}

/// Bookkeeping vector for iterative approach
type UndoStack = Vec<UndoStackElement>;