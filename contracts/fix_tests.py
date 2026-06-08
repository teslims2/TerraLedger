import os, glob

for fpath in glob.glob("/home/jemiah/Documents/TerraLedger/TerraLedger/contracts/*/src/lib.rs"):
    with open(fpath, "r") as f:
        content = f.read()
    
    # We will remove all env.mock_all_auths()
    content = content.replace("        env.mock_all_auths();\n", "")
    content = content.replace("        env.mock_all_auths();", "")
    
    # And insert it right after Carbon.*Client::new
    lines = content.split('\n')
    new_lines = []
    for line in lines:
        new_lines.append(line)
        if "Client::new(" in line:
            new_lines.append("        env.mock_all_auths();")
    
    with open(fpath, "w") as f:
        f.write('\n'.join(new_lines))
