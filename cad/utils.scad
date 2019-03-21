module rounded_square(size=1, center=false, r=1) {
  offset(r)
    offset(-r)
      square(size, center=center);
}

module chamfered_pocket(size=1, h=1) {
  intersection() {
    hull() {
      translate([0,0,h/2])
        cube([size.x, size.y, h], center=true);

      translate([0,0,1.5*h])
        cube([size.x+h*2, size.y+h*2, h], center=true);
    }
    translate([0,0,0.5*h])
      cube([size.x+h*2, size.y+h*2, h], center=true);
  }
}
