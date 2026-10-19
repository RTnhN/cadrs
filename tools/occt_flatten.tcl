# Runs OCCT test scripts (DRAW Tcl) with DRAW's commands logged instead of run, so loops,
# procedures, variables and substitutions are expanded: `tclsh occt_flatten.tcl file...` sources
# the files in order and prints each DRAW command it meets as `CMD <Tcl list>`.
#
# DRAW commands return nothing here: a script branching on a command's output takes whichever
# way an empty result leads, and one using it in arithmetic fails (the converter then reports
# the case as needing evaluation). `locate_data_file` returns `@data:<name>` for the converter.

rename puts _puts
proc puts {args} {}
proc emit {args} { _puts stdout "CMD [list {*}$args]" }

# Any command Tcl doesn't know is a DRAW command: log it.
proc unknown {args} {
    emit {*}$args
    return ""
}

proc locate_data_file {name} { return "@data:$name" }
proc isdraw {args} { return 1 }
proc cpulimit {args} {}
proc pload {args} {}
proc decho {args} {}
proc dchrono {args} {}
proc chrono {args} {}
proc meminfo {args} { return 0 }
proc checkreal {args} {}
proc checktrinfo {args} {}
proc checkview {args} {}
proc checkmaxtol {args} {}
proc checksection {args} {}
proc checkfreebounds {args} {}
proc checkcolor {args} {}
proc dlog {args} { return "" }

array set Draw_Groups {}
set imagedir .
set test_image photo
set casename case
set groupname group
set gridname grid
set dirname .

foreach f $argv {
    source $f
}
