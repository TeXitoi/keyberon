use <utils.scad>

module key() {
    linear_extrude(10, scale=0.7)
        rounded_square([18.5,18.5], r=3, center=true);
}
