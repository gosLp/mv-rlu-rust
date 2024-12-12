import pandas as pd
import matplotlib.pyplot as plt
import re

file_path = "logfile.csv"  
df = pd.read_csv(file_path)

# Function to plot Thread Count vs. Throughput for a given Write Percent
def plot_over_threadcount(write_percent):
    filtered = df[df["Write Percent"] == write_percent]
    if filtered.empty:
        print(f"No data for Write Percent = {write_percent}")
        return
    plt.figure(figsize=(10, 6))
    plt.plot(filtered["Thread Count"], filtered["Throughput"], marker="o")
    plt.title(f"Thread Count vs. Throughput at Write Fraction = {write_percent}")
    plt.xlabel("Thread Count")
    plt.ylabel("Throughput")
    plt.grid()
    plt.savefig(f"plots/thread_throughput_{write_percent}.png")


# Function to plot Write Percent vs. Thread Count for a given Thread Count
def plot_over_write(thread_count):
    filtered = df[df["Thread Count"] == thread_count]
    if filtered.empty:
        print(f"No data for Thread Count = {thread_count}")
        return
    plt.figure(figsize=(10, 6))
    plt.plot(filtered["Write Percent"], filtered["Throughput"], marker="o")
    plt.title(f"Write Fraction vs. Throughput at Thread Count = {thread_count}")
    plt.xlabel("Write Fraction")
    plt.ylabel("Throughput")
    plt.grid()
    plt.savefig(f"plots/write_throughput_{thread_count}.png")

plot_over_threadcount(0.0)
plot_over_threadcount(0.02)
plot_over_threadcount(0.05)
plot_over_threadcount(0.1)
plot_over_threadcount(0.2)
plot_over_threadcount(0.3)
plot_over_threadcount(0.4)

plot_over_write(2)
plot_over_write(4)
plot_over_write(8)
plot_over_write(16)
plot_over_write(20)
plot_over_write(30)
plot_over_write(40)