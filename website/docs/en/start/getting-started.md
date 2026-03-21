# Getting Started

SillyStage is a multi-agent system for AI interactive storytelling. You can think of it as a runnable narrative engine: you provide characters, setting, story resources, and model configuration, and the system coordinates planning, directing, acting, narration, and state updates across each session turn.

This guide walks through one end-to-end example: start the service, finish the base configuration, create a story, enter a session, and run the first interaction.

## 1. Install and Boot

- Install and start SillyStage. See the [Installation Guide](./installation-guide).
- Default listen address: `http://127.0.0.1:8080`
- If automatic browser opening is enabled, a browser window opens after startup
- Otherwise, open the local address manually

## 2. Configure the API

Prepare a usable LLM API first. Only OpenAI-compatible APIs are currently supported.

1. Open the API management page and create an API entry with your credentials and model configuration.
2. Create an API group and assign that API to the agents you want to run.

## 3. Prepare Basic Story Resources

1. Configure presets

   Presets define agent parameters such as temperature and prompt modules.

   Open the Presets page and use the sample action to create a base preset.

2. Optional: Configure schemas

   Schemas declare which variables a story uses, what they mean, and what types they have. They help drive story transitions and provide references for agent responses.

   Open the Schemas page and use the sample action to create a starter schema.

3. Optional: Configure lorebooks

   Lorebooks add background details such as scenes, history, and setting information. They help keep character behavior and dialogue more coherent.

   Open the Lorebooks page and use the sample action to generate a starter lorebook.

4. Optional: Configure player profiles

   Player profiles add more detail for the player character.

   You can create a sample player profile in the same way.

5. Add character cards

   Character cards are one of the main building blocks of a story. They describe a character's personality, background, and characteristic lines.

   The sample setup provides three character cards. Add or replace them as needed.

## 4. Create Story Resources

Story resources are the main input for story generation. One story resource can be used to generate multiple stories.

Open the Story Resources page and click the create button in the top-right corner to enter the creation wizard.

1. Enter the raw story input

   The raw story input gives the Planner agent source material it can organize into a script that is easier for the Architect agent to turn into a story graph.

   If you do not want Planner assistance, you can enter the full script directly here.

2. Choose character cards

   Select the characters that should participate in the story.

3. Optional: Set initial schemas and lorebooks

   Initial schemas and lorebooks are optional, but they can improve world detail and runtime behavior.

   An initial schema gives the Architect agent a variable reference so the generated schema structure better matches your needs.

4. Organize the story

   If you want the Planner agent to refine the raw input, choose Create and send to Planner. Otherwise choose Save raw input only.

   In most cases, sending it to Planner gives better results.

## 5. Create the Story

Open the Stories page and click the create button in the top-right corner to start story creation.

Stories can be created manually or generated from resources. This guide only covers generation from story resources.

1. Choose the API group, preset, and story name

2. Optional: Set pinned common variables

   These variables stay visible in the right-side panel of the stage page for quick reference.

3. Start story generation

   Story generation can take a while, and failures may happen depending on the LLM you use.

   If generation stops with an error, open the draft page and continue the interrupted flow from there.

## 6. Enter a Session

Switch to the stage page and click the new-session button in the left sidebar. Select the configuration you want, then start the dialogue flow.
