# Local Registry Schema

Each discovered agent record should contain:

```json
{
  "id": "cmdc",
  "display_name": "Command Code",
  "status": "ready",
  "command": "cmdc",
  "path": "C:/.../cmdc.ps1",
  "checks": {
    "help": true,
    "version": true,
    "models": true,
    "status": true
  },
  "capabilities": ["code-implementation", "code-review", "long-context-synthesis", "vision"],
  "strengths": ["model-rich", "good for synthesis"],
  "weaknesses": ["depends on account/model access"],
  "recommended_tasks": ["vision", "long-context-synthesis", "review"],
  "notes": []
}
```
