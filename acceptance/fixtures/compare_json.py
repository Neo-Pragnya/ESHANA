import json, sys
a, b = (json.load(open(p)) for p in sys.argv[1:3])
if a != b:
    print("DIFFERENT")
    sys.exit(1)
print("IDENTICAL")
