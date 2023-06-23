# run a terminal with: . setup.sh
sudo mkdir -p /fsx && sudo mount -t lustre -o noatime,flock fs-0e6759f50ff7a310c.fsx.us-west-2.amazonaws.com@tcp:/x346hbmv /fsx
export FEM_REPO=/fsx/20230131_1605_zen_30_M1_202110_ASM_202208_Mount_202111
export CFD_REPO=/fsx/CASES
export GMT_MODES_PATH=/fsx/ceo
export N_KL_MODE=496
export ZA=30
export AZ=0
export VS=os
export WS=7
export RUST_LOG=info