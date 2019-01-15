extern crate scoped_threadpool;
extern crate glob;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::result::Result;
use std::env;
use scoped_threadpool::Pool;
use glob::glob;

/*
 * Given a string, count the number of 
 * words within it.
 */
fn count_words( s: &String ) -> i32 {

    let mut words: i32 = 0;

    for c in s.chars() {

        if c.is_whitespace() {
            words += 1;    
        }
        
    }

    words + 1
}

// Static dispatch. 
// Here we specify that count_words works for
// objects with the BufRead trait only.
fn counter<R: BufRead> ( reader: &mut R ) -> Result<( i32, i32 ), String> {

    // Define our line and word count variables
    let mut total_lines: i32 = 0;
    let mut total_words: i32 = 0;

    // Create a String. This will be where each line is read to
    let mut line = String::from( "" );

    // Start a loop
    loop{ 

        // Attempt to read a line into 'line'
        match reader.read_line( &mut line ) {

            // We successfully read some bytes
            Ok( _ ) => {

                    // Exit loop if we didn't read any data.
                    if line.len() == 0 {
                        break;
                    }

                    // Trim the string and increment lines & words
                    line = line.trim().to_string();
                    total_lines += 1;
                    total_words += count_words( &line );
                    // Clear the string buffer
                    line.clear(); 
            },
            // If an error occurred, return it early
            Err( why ) => return Err( why.to_string() )
        };
    

    }
    // Return the counts if everything went ok.
    Ok( ( total_lines, total_words ) )
}

/*
 * Given a file, create a buffered reader, and
 * start reading & counting the data.
 */
fn count_file( file_path: &Path ) -> Result< (i32,i32), String> {

    // Attempt to open the file, returning an Err()
    // or continuing with the file_handle
    let file_handle = match File::open( &file_path ) {
        Err( why ) => return Err( why.to_string() ),
        Ok( file_handle ) => file_handle
    };

    // On successful opening of the file, create a buffered reader
    let mut reader = BufReader::new( file_handle );

    // Call the counter and return the results.
    let ( lines, words ) = counter( &mut reader )?;
    Ok( ( lines, words ) )

}

/*
 * Main
 *
 * I spent a lot of time struggling with lifetimes here.
 * Threads can exist well after a scope has ended. This
 * caused me huge headaches trying to pass in values from
 * an iterator, because these values could go out of scope
 * before the thread finishes.
 *
 * Solution - scoped threads using scoped_threadpool.
 * These guarantee an exit before a scope ends.
 *
 * Ref:
 * https://crates.io/crates/scoped_threadpool
 * https://stackoverflow.com/questions/32750829/how-can-i-pass-a-reference-to-a-stack-variable-to-a-thread
 *
 */
fn main() {

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Set themax number of workers
    let max_workers = 4;

    // Determine if we have any arguments. There will be a MUCH
    // better way to do this.
    if args.len() < 2 {
        panic!( "Program arguments missing. Please provide a file name" );            
    }

    // Get arguments from the command line, skipping the program name
    let files: Vec<String> = Vec::from( &args[1..] );

    // Create a pool of worker threads
    let mut pool = Pool::new( max_workers );

    // Create a scope
    pool.scoped( |scoped| {            
                                  
            // Iterate through file names. 
            for file_arg in files.iter() {     

                // Match the glob pattern, filtering out bad paths
                for file_name in glob(file_arg).unwrap().filter_map(Result::ok) {                    

                    // Fire-off a worker thread
                    scoped.execute( move || {
                            let path = Path::new( &file_name ); 
                            
                            // Execute count_file() on it, parsing the response.
                            match count_file( path ) {
                                // Parse the result
                                Ok( ( lines, words ) ) => {
                                    println!("{}\t{} lines\t{} words.", path.display(), lines, words );
                                },
                                Err( err ) => {
                                    panic!("Error - {}", err );
                                }

                            };
                        } 
                    ); //scoped.execute
                }
            } //for
        }
    ); //pool.scoped
}