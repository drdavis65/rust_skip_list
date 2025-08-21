#!/bin/bash

# Configuration
NUM_RUNS=${1:-10}
BINARIES_DIR="."
RESULTS_DIR="benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create results directory
mkdir -p $RESULTS_DIR

# CSV headers for results (added opt_level column)
echo "filename,opt_level,run,runtime_ms,l1_dcache_misses,l1_icache_misses,cache_misses,branch_misses,instructions,cycles" > $RESULTS_DIR/timing_results_${TIMESTAMP}.csv

# Function to clear cache
clear_cache() {
    echo "Clearing cache..."
    sync
    echo 3 | sudo tee /proc/sys/vm/drop_caches > /dev/null
    sleep 1
}

# Function to extract optimization level from filename
extract_opt_level() {
    local filename=$1
    # Look for pattern like "O0", "O1", "O2", "O3" in the filename
    if [[ $filename =~ _O([0-3])$ ]]; then
        echo "O${BASH_REMATCH[1]}"
    else
        echo "N/A"
    fi
}

# Function to clean filename by removing skip_list_ prefix and _OX suffix
clean_filename() {
    local filename=$1
    # Remove skip_list_ prefix if present
    cleaned=${filename#skip_list_}
    # Remove _OX suffix if present (where X is 0-3)
    cleaned=${cleaned%_O[0-3]}
    echo "$cleaned"
}

# Function to run benchmark
run_benchmark() {
    local binary_path=$1
    local filename=$2
    local run_num=$3
    
    # Clean filename for display
    clean_name=$(clean_filename "$filename")
    
    echo "Running: $clean_name (Run $run_num/$NUM_RUNS)"
    
    # Extract optimization level from filename
    opt_level=$(extract_opt_level "$filename")
    # Clean filename for display/CSV
    clean_name=$(clean_filename "$filename")
    
    # Clear cache before each run
    clear_cache
    
    # Temporary files for outputs
    perf_output=$(mktemp)
    program_output=$(mktemp)
    
    # Run with perf stat to collect performance counters
    perf stat -e L1-dcache-misses,L1-icache-misses,cache-misses,branch-misses,instructions,cycles \
    -o $perf_output $binary_path > $program_output 2>&1
    
    # Extract runtime from program output
    runtime_ms="N/A"
    if [[ -f $program_output ]]; then
        # Look for "Total time: XXX ms" pattern (updated for skip list output)
        runtime_line=$(grep "Total time:" $program_output)
        if [[ -n "$runtime_line" ]]; then
            # Extract the number before "ms"
            runtime_ms=$(echo "$runtime_line" | sed -n 's/.*Total time: \([0-9]\+\) ms.*/\1/p')
        fi
        
        # Fallback: also check for "Processing time: XXXms" pattern (for other programs)
        if [[ "$runtime_ms" == "N/A" ]]; then
            runtime_line=$(grep "Processing time:" $program_output)
            if [[ -n "$runtime_line" ]]; then
                runtime_ms=$(echo "$runtime_line" | sed -n 's/.*Processing time: \([0-9]\+\)ms.*/\1/p')
            fi
        fi
    fi
    
    # Parse perf results
    if [[ -f $perf_output ]]; then
        l1_dcache_misses=$(grep "L1-dcache-misses" $perf_output | awk '{print $1}' | sed 's/,//g')
        l1_icache_misses=$(grep "L1-icache-misses" $perf_output | awk '{print $1}' | sed 's/,//g')
        cache_misses=$(grep -w "cache-misses" $perf_output | awk '{print $1}' | sed 's/,//g')
        branch_misses=$(grep -w "branch-misses" $perf_output | awk '{print $1}' | sed 's/,//g')
        instructions=$(grep -w "instructions" $perf_output | awk '{print $1}' | sed 's/,//g')
        cycles=$(grep -w "cycles" $perf_output | awk '{print $1}' | sed 's/,//g')
        
        # Handle cases where perf couldn't measure or values are empty
        [[ -z "$l1_dcache_misses" || "$l1_dcache_misses" == "<not" ]] && l1_dcache_misses="N/A"
        [[ -z "$l1_icache_misses" || "$l1_icache_misses" == "<not" ]] && l1_icache_misses="N/A"
        [[ -z "$cache_misses" || "$cache_misses" == "<not" ]] && cache_misses="N/A"
        [[ -z "$branch_misses" || "$branch_misses" == "<not" ]] && branch_misses="N/A"
        [[ -z "$instructions" || "$instructions" == "<not" ]] && instructions="N/A"
        [[ -z "$cycles" || "$cycles" == "<not" ]] && cycles="N/A"
    else
        l1_dcache_misses="ERROR"
        l1_icache_misses="ERROR"
        cache_misses="ERROR"
        branch_misses="ERROR"
        instructions="ERROR"
        cycles="ERROR"
    fi
    
    # Write to CSV (using cleaned filename)
    echo "$clean_name,$opt_level,$run_num,$runtime_ms,$l1_dcache_misses,$l1_icache_misses,$cache_misses,$branch_misses,$instructions,$cycles" >> $RESULTS_DIR/timing_results_${TIMESTAMP}.csv
    
    # Cleanup temp files
    rm -f $perf_output $program_output
}

# Check if running as root/sudo for cache clearing
if [[ $EUID -ne 0 ]]; then
    echo "Note: This script needs sudo privileges to clear cache between runs."
    echo "You may be prompted for your password."
fi

echo "========== Starting Performance Benchmarking =========="
echo "Results will be saved to: $RESULTS_DIR/timing_results_${TIMESTAMP}.csv"
echo "Number of runs per executable: $NUM_RUNS"
echo "Binaries directory: . (current directory)"
echo ""

# Find all executable files in the binaries directory, excluding .sh files
executables=($(find "$BINARIES_DIR" -type f -executable ! -name "*.sh"))

if [[ ${#executables[@]} -eq 0 ]]; then
    echo "Error: No executable files found in current directory (excluding .sh files)"
    exit 1
fi

echo "Found ${#executables[@]} executable(s):"
for exe in "${executables[@]}"; do
    filename=$(basename "$exe")
    opt_level=$(extract_opt_level "$filename")
    clean_name=$(clean_filename "$filename")
    echo "  - $clean_name (opt level: $opt_level)"
done
echo ""

# Run benchmarks for each executable
for executable in "${executables[@]}"; do
    filename=$(basename "$executable")
    clean_name=$(clean_filename "$filename")
    echo "Benchmarking: $clean_name"
    
    for run in $(seq 1 "$NUM_RUNS"); do
        run_benchmark "$executable" "$filename" "$run"
    done
    
    echo "Completed all runs for $clean_name"
    echo ""
done

echo "========== Benchmarking Complete =========="
echo "Results saved to: $RESULTS_DIR/timing_results_${TIMESTAMP}.csv"
echo ""
echo "Summary of collected data:"
echo "- Filename of executable"
echo "- Optimization level (0, 1, 2, 3, or N/A)"
echo "- Run number"
echo "- Runtime extracted from program output (milliseconds)"
echo "- L1 data cache misses"
echo "- L1 instruction cache misses" 
echo "- Cache misses"
echo "- Branch misses"
echo "- Instructions executed"
echo "- CPU cycles"
echo ""
echo "You can analyze the results using:"
echo "  cat $RESULTS_DIR/timing_results_${TIMESTAMP}.csv"