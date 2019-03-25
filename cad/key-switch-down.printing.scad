use <key.scad>
include <printing.scad>

rotate([180,0,0]) {
    switch();
    key(down=true);
}
