case $1 in
  "none") exts="" ;;
  ""|"24"|"G24"|"g24") exts=G24 ;;
  "8"|"GRY"|"gry") exts=GRY ;;
  "both"|"all") exts="G24 GRY" ;;
  *) echo "Invalid type $1, want 'G24' or 'GRY' or 'both'" ;;
esac

test_command1() {
  cmd=$1
  file=$2
  basefile=$(basename $2)

  echo " Testing ${cmd} ${file}"

  gotfile=testdata/got_${cmd}_${basefile}.txt
  wantfile=testdata/want_${cmd}_${basefile}.txt

  cargo run --quiet -- ${cmd} ${file} > ${gotfile}
  if ! diff ${gotfile} ${wantfile}
  then
    echo "ERROR: ${gotfile} != ${wantfile}"
    exit 1
  fi
}

########
# info #
########
test_command1 info gamedata/gta-uk/UK.CMP
test_command1 info gamedata/gta-uk/MC.CMP
test_command1 info gamedata/gta/MIAMI.CMP
test_command1 info gamedata/gta/NYC.CMP
test_command1 info gamedata/gta/SANB.CMP

###########
# extract #
###########

###########
# overview #
###########

###########
# display #
###########

for e in $exts
do
  case $1 in
    ""|"24"|"G24"|"g24") EXT=G24 ; LC_EXT=g24 ;;
    "8"|"GRY"|"gry") EXT=GRY ; LC_EXT=gry ;;
    *) echo "Invalid type $1, want G24 or GRY" ;;
  esac

  # Houses roof in London (shouldn't be transparent)
  cargo run -- display gamedata/gta-uk/UK.CMP gamedata/gta-uk/Style001.${LC_EXT} --camera-position 212.59,6.07,263.88 --camera-rotation 2.04,-26.48,-0.00

  # London Bridge (should be transparent)
  cargo run -- display gamedata/gta-uk/UK.CMP gamedata/gta-uk/Style001.${LC_EXT} --camera-position 248.89,14.81,153.32 --camera-rotation 35.88,-31.00,0.00

  # Hospital sign in London
  cargo run -- display gamedata/gta-uk/UK.CMP gamedata/gta-uk/Style001.${LC_EXT} --camera-position 199.45,6.07,204.78 --camera-rotation 91.83,-24.43,0.00

  # Docks sign, transparent lids and animated sides (GIRLS) in Liberty City
  cargo run -- display gamedata/gta/NYC.CMP gamedata/gta/STYLE001.${EXT} --camera-position 234.69,2.46,54.64 --camera-rotation 97.08,-31.54,0.00

  # Water and shades in London
  cargo run -- display gamedata/gta-uk/UK.CMP gamedata/gta-uk/Style001.${LC_EXT} --camera-position 100.18,6.07,169.29 --camera-rotation 47.86,-26.48,0.00

  # Water and shades in Liberty City
  cargo run -- display gamedata/gta/NYC.CMP gamedata/gta/STYLE001.${EXT} --camera-position 234.69,2.46,54.64 --camera-rotation 161.54,-28.38,-0.00

  # Animated sides in London (Piccadilly circus)
  cargo run -- display gamedata/gta-uk/UK.CMP gamedata/gta-uk/Style001.${LC_EXT} --camera-position 88.81,6.07,52.15 --camera-rotation " -32.82,-26.48,0.00"

  # Pool animated side in Liberty City
  cargo run -- display gamedata/gta/NYC.CMP gamedata/gta/STYLE001.${EXT} --camera-position 120.69,2.87,113.27 --camera-rotation 46.23,-27.58,-0.00

  # San Andreas bridge view
  cargo run -- display gamedata/gta/SANB.CMP gamedata/gta/STYLE002.${EXT} --camera-position 247.16,15.65,119.50 --camera-rotation 151.77,-26.75,0.00

  # Vice City random view
  cargo run -- display gamedata/gta/MIAMI.CMP gamedata/gta/STYLE003.${EXT} --camera-position 201.47,15.65,119.57 --camera-rotation 18.83,-26.75,-0.00

  # Buckingham palace and building in front of it (which should have black sides, not transparent sides)
  cargo run -- display gamedata/gta-uk/UK.CMP gamedata/gta-uk/Style001.${LC_EXT} --camera-position 26.58,6.25,170.25 --camera-rotation " -30.76,-22.56,0.00"
done
