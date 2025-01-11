import wezpy
import asyncio


c = wezpy.WeztermClient()

async def main():
    id = await c.find_pane(
        workspace_pattern="wezpy",
        # tab_pattern="ninjas",
        title_pattern="> hx$"
    )

    print(id)

    

asyncio.run(main())
