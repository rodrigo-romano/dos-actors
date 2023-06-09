# WIND LOADING

Applies wind loads to the mount and M1 and M2 segments and saves M1 and M2 rigid body motion time series.

The FEM should include the mount FEM that correspond to the final design with the control system to match.

The following environment variables are used:
 * FEM_REPO: sets to the path to the FEM model
 * CFD_REPO: sets to the path to the CFD data
 * DATA_REPO: sets the path to where the data is saved to (if not set the data is saved in the current directory)