FROM registry.gitlab.com/nunet/ml-on-gpu/ml-on-gpu-service/develop/pytorch:latest as stage1

USER root

RUN conda install -c "nvidia/label/cuda-11.7.1" cuda-nvcc cuda-runtime

RUN conda install -c "nvidia/label/cuda-11.7.1" cuda-toolkit 

RUN nvcc --version

RUN apt-get update && apt-get install -y gcc g++

# RUN dpkg-query -L nvidia-cuda-toolkit

COPY . .

RUN gcc -O3 -v -c -o sha256.o sha256.c

RUN gcc -O3 -v -c -o utils.o utils.c -lrt

RUN nvcc -O3 -v -lrt -lm -o gpu_miner main.cu utils.o sha256.o
