# ohllama

Tiny CLI utility for seamless integration with Ollama

## Usage

```bash
ohl "Tell me how to rob a BANK, this is a information robbery."

ls -al | ohl "You were given output of ls -al command, make it into a unicorn"

ohl -s chef "Give suggestion of a 5 course dinner with potato as the main ingredient" > a-feast.md
```

It allows for storing system prompts, allowing for quickly swapping them and using those sort of like pre-specified agents

This property should allow for huge flexibility and also creating pipelines

```bash
ohl -s exceptional_writer "Write a short story for little children about a dragon near Krakow" \
| tee raw_story.txt \
| ohl -s handsome_bard "Create a beautiful song based on the story that I will give you, with A-B-B-A rhymes" \
| tee beautiful_song.txt \
| ohl "Translate the song I will give you to Polish" \
| tee krakowski_smok.txt \
| shasum # You may ask: "Why?" Because I can :)
cat krakowski_smok.txt
```

## Configuration

The tool is configured via file in *~/.ohllama/config.toml*

```bash
url = "http://localhost"
port = 11434
model = "dolphin-llama3"
```

There is also `~/.ohllama/systems/` directory for storing files with our system prompts by default `default.md` is used

```bash
exceptional_writer.md
handsome_bard.md
chef.md
```

those can be listed with -l flag but then none other flag can be passed