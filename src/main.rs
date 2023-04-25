mod udev_util;
mod randr_util;

use std::collections::HashSet;

use udev_util::Udev;
use randr_util::Randr;

fn main() {
    println!("Hello, world!");
    let randr = Randr::new();

    let mut active_outputs = randr.get_active().unwrap();
    let mut new_active_outputs;

    println!("Active outputs: {:#?}", active_outputs);

    let udev = Udev::new();
    loop {
        udev.wait();
        println!("UDEV!");

        new_active_outputs = randr.get_active().unwrap();
        let removed_outputs = active_outputs.difference(&new_active_outputs);
        let added_outputs = new_active_outputs.difference(&active_outputs);

        println!("Added outputs: {:#?}", added_outputs);
        println!("Removed outputs: {:#?}", removed_outputs);

        for o in removed_outputs {
            randr.turn_off(o).unwrap();
        }

        for o in added_outputs {
            randr.set_best_mode(o).unwrap();
        }

        active_outputs = new_active_outputs;




        // TODO get list of active outputs
        // identify new outputs
        //   auto config new outputs

        // OLD STUFF
        /*
        for o in res {
            if o.output == 84 {
                println!("set best mode");
                randr.set_best_mode(&o).unwrap();
            }
        }

        let res = randr.get().unwrap();
        println!("displays {res:#?}");
        */
    }
}
