## Usage

```sh
❯ cargo r -p simple-runtime -- -i path/to/input -o path/to/output
❯ ./runtime -i path/to/input -o path/to/output

Options:
  -i, --input <INPUT>    Input file or directory
  -o, --output <OUTPUT>  Output directory
  -c, --config <CONFIG>  Optional config file
  -v, --verbose...       Verbose mode (-v, -vv, -vvv)
      --overwrite        Overwrite already translated images
  -h, --help             Print help
  -V, --version          Print version
```

Only 
- coreml
- cuda
- cpu
- tensorrt

is supported right now. For AMD support look at how to enable rocm for onnxruntime or maybe ZLUDA
