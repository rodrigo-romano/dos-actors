#include "solver.hpp"

// state equation`
__global__ void step_kernel(mode_state_space *mss, double *v, double *y,
                            int n) {
  int i;
  double x0, x1;
  i = blockIdx.x * blockDim.x + threadIdx.x;
  if (i < n) {
    x0 = mss[i].x0;
    x1 = mss[i].x1;
    // x0 <- a0 x0 + a2 x1 + b2 vi
    mss[i].x0 = mss[i].a0 * x0 + mss[i].a2 * x1 + mss[i].b2 * v[i];
    // x1 <- a1 x0 + a3 x1 + b3 vi
    mss[i].x1 = mss[i].a1 * x0 + mss[i].a3 * x1 + mss[i].b3 * v[i];
    y[i] = mss[i].x0;
  }
}

void state_space::build(int n_mode_, mode_state_space *mss, int n_input_,
                        double *i2m, int n_output_, double *m2o) {

  n_mode = n_mode_;
  n_input = n_input_;
  n_output = n_output_;
  d_dcg = NULL;
  cublasCreate(&handle);
  cudaMalloc(&d_mss, n_mode * sizeof(mode_state_space));
  cudaMemcpy(d_mss, mss, n_mode * sizeof(mode_state_space),
             cudaMemcpyHostToDevice);
  cudaMalloc(&d_i2m, n_mode * n_input * sizeof(double));
  cudaMemcpy(d_i2m, i2m, n_mode * n_input * sizeof(double),
             cudaMemcpyHostToDevice);
  cudaMalloc(&d_m2o, n_mode * n_output * sizeof(double));
  cudaMemcpy(d_m2o, m2o, n_mode * n_output * sizeof(double),
             cudaMemcpyHostToDevice);
  cudaMalloc(&d_u, n_input * sizeof(double));
  cudaMalloc(&d_v, n_mode * sizeof(double));
  cudaMalloc(&d_x0, n_mode * sizeof(double));
  cudaMalloc(&d_y, n_output * sizeof(double));
}
void state_space::dc_gain_compensator(double *dcg) {
  cudaMalloc(&d_dcg, n_output * n_input * sizeof(double));
  cudaMemcpy(d_dcg, dcg, n_output * n_input * sizeof(double),
             cudaMemcpyHostToDevice);
}
void state_space::free() {
  cublasDestroy(handle);
  cudaFree(d_mss);
  cudaFree(d_i2m);
  cudaFree(d_m2o);
  cudaFree(d_u);
  cudaFree(d_v);
  cudaFree(d_x0);
  cudaFree(d_y);
  if (d_dcg != NULL)
    cudaFree(d_dcg);
}
void state_space::step(double *u, double *y) {
  double alpha = 1.0;
  double beta = 0.0;
  dim3 block(256); // or whatever block size you want
  dim3 grid((n_mode + block.x - 1) / block.x); // ceil(n/block.x)

  cudaMemcpy(d_u, u, n_input * sizeof(double), cudaMemcpyHostToDevice);
  // v = Bu
  cublasDgemv(handle, CUBLAS_OP_T, n_input, n_mode, &alpha, d_i2m, n_input, d_u,
              1, &beta, d_v, 1);

  // update state equation
  step_kernel<<<grid, block>>>(d_mss, d_v, d_x0, n_mode);

  // y = Cx0
  cublasDgemv(handle, CUBLAS_OP_N, n_output, n_mode, &alpha, d_m2o, n_output,
              d_x0, 1, &beta, d_y, 1);
  cudaMemcpy(y, d_y, n_output * sizeof(double), cudaMemcpyDeviceToHost);

  if (d_dcg != NULL) {
    beta = 1.0;
    cublasDgemv(handle, CUBLAS_OP_N, n_output, n_input, &alpha, d_dcg, n_output,
                d_u, 1, &beta, d_y, 1);
  }
  cudaMemcpy(y, d_y, n_output * sizeof(double), cudaMemcpyDeviceToHost);
}
