all: left-case.stl right-case.stl left-back.stl right-back.stl

%.stl: %.printing.scad case.scad printing.scad utils.scad
	openscad -o $@ $<
