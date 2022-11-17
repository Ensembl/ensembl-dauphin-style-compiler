#! /usr/bin/env python3

import re, sys, getopt, shlex, json

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

# OPTIONS

use_rich = sys.stdout.isatty() 
use_prev = False
quick = False

(optlist,args) = getopt.getopt(sys.argv[1:],[],['rich','no-rich','use-prev=','quick'])
for (option,value) in optlist:
    if option == '--rich':
        use_rich = True
    elif option == '--no-rich':
        use_rich = False
    elif option == '--use-prev':
        use_prev = value
    elif option == '--quick':
        quick = True

if len(args) != 2:
    eprint("Syntax {0} [options] config-file.json output-file.sh".format(sys.argv[0]))
    sys.exit(1)

config_file = args[0]
output_file = args[1]

# RICH TERMINAL

CODES = {
    'X': '\033[2J\033[H'
}

COLOUR_CODES = { 'r': '91;1', 'g': '32;1', 'y': '93;1', '-': '39;0', 'b': '34;1', 'c': '96;1' }
CODES |= { k: "\033[{0}m".format(v) for (k,v) in COLOUR_CODES.items() }

if not use_rich:
    CODES = { k: '' for k in CODES.keys() }

def rich(line):
    return re.sub(r'\0(.)',lambda x: CODES[x[1]],line)

# PREVIOUS VALUES
defaults = {}
if use_prev:
    try:
        with open(use_prev,'r') as f:
            defaults = json.load(f)
    except:
        pass

# QUESTION ASKING

def unique(a,options):                                                                  
  a = a.strip().lower()
  cands = set()
  for option in options:
    if option.startswith(a):
      cands.add(option)
  if len(cands) == 1:
    return cands.pop()
  else:
    return None

class VerifyNumber:
    def __init__(self,min,max):
        self.min = min
        self.max = max

    def verify(self,value):
        try:
            number = int(value)
            if self.min != None and number < self.min:
                return "Too small, must be at least {0}".format(self.min)
            if self.max != None and number > self.max:
                return "Too big, must be no more than {0}".format(self.max)
            return None
        except:
            return "Not a number"

class Free:
    def __init__(self, question, default):
        self.question = question
        self.default = default

    def label(self):
        return self.question

    def ask(self, default, skip_ask):
        if default is None:
            default = self.default
        if skip_ask:
            return default
        print(rich("\0y{0}\0- [\0g{1}\0-]? ".format(self.question,default)),end='',flush=True)
        line = sys.stdin.readline().strip()
        if line == '':
            return default
        else:
            return line

class ChooseOne:
    def __init__(self, question, options):
        self.question = question
        self.options = options[:]

    def label(self):
        return self.question

    def ask(self, default, skip_ask):
        options = self.options[:]
        if default != None:
            try:
                index = options.index(default)
                options.insert(0,options.pop(index))
            except:
                pass
        if skip_ask:
            return options[0]
        optstr = "/".join(["\0{0}{1}\0-".format("g" if i == 0 else "y",x) for (i,x) in enumerate(options)])
        while True:
            print(rich("\0y{0}\0- [{1}]? ".format(self.question,optstr)),end='',flush=True)
            line = sys.stdin.readline().strip()
            if line != '': 
                a = unique(line,options)
                if a != None:
                    return a
                else:
                    print(rich("\0rEh?\0- Please type one of: {0}".format(", ".join(self.options))))
            else:
                return options[0]

def conditions_met(conditions,values):
    if "eq" in conditions:
        if values.get(conditions["eq"][0],None) != conditions["eq"][1]:
            return False
    return True

def ask(question,verifiers,default,skip_ask,feedback=True):
    while True:
        out = question.ask(default,skip_ask)
        error = None
        for verifier in verifiers:
            error = verifier.verify(out)
            if error != None:
                print(rich("\0rProblem:\0- {0}".format(error)))
                break
        if error == None:
            if feedback:
                print(rich("Ok, using \0c{0}\0-\n".format(out)))
            return out
        print("\n")

def ask_all(questions):
    first = True
    while True:
        skip_ask = (quick and first)
        first = False
        out = {}
        # Ask
        for q in questions:
            default = defaults.get(q[0],None)
            if conditions_met(q[3],out):
                out[q[0]] = ask(q[1],q[2],default,skip_ask)
            else:
                out[q[0]] = ask(q[1],"",None,True,feedback=False)

        # Show settings for confirmation
        print(rich("\0X\0gSummary:\0-"))
        for q in questions:
            if conditions_met(q[3],out):
                print(rich("{0}: \0c{1}\0-".format(q[1].label(),out[q[0]])))
        print("\n")

        # Confirm
        confirm = ChooseOne("Are these ok?",["yes","no","quit"]).ask(None,False)
        if confirm == "quit":
            sys.exit(1)
        elif confirm == "yes":
            break
        else:
            print(rich("\0X"))
    return out


# Read the config

config = []
with open(config_file,'r') as f:
    config_json = json.load(f)
    for prompt in config_json:
        verifiers = []
        for verifier in prompt.get("verifiers",[]):
            if verifier["verifier"] == "number":
                verifiers.append(VerifyNumber(verifier.get("min",None),verifier.get("max",None)))
        conditions = prompt.get("conditions",{})
        if "options" in prompt:
            config.append([prompt["key"],ChooseOne(prompt["question"],prompt["options"]),verifiers,conditions]),
        else:
            config.append([prompt["key"],Free(prompt["question"],prompt.get("default","")),verifiers,conditions])

print(rich("\0X\0yConfiguration\0-\nFor default (\0ggreen\0-) hit enter. Unambiguous prefixes are fine. Defaults usually sensible.\n"))

# do the asking

answers = ask_all(config)

# write file
with open(output_file,'w') as f:
    for (key,value) in answers.items():
        f.write("{0}='{1}'\n".format(key,shlex.quote(value)))

if use_prev:
    with open(use_prev,'w') as f:
        json.dump(answers,f)

print()
