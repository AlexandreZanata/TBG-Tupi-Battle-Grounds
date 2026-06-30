using System;

namespace TBG.CharacterLibrary
{
    [Serializable]
    public sealed class BehaviorActionsRecord
    {
        public string idle;
        public string spawn;
        public string attack;
        public string death;
    }

    [Serializable]
    public sealed class BehaviorTriggersRecord
    {
        public string idle;
        public string spawn;
        public string attack;
        public string death;
    }

    [Serializable]
    public sealed class BehaviorTimingsRecord
    {
        public int attackWindupMs;
        public int attackHitMs;
    }

    [Serializable]
    public sealed class BehaviorProfileJsonRecord
    {
        public string id;
        public string archetype;
        public string bodyClass;
        public string animatorController;
        public string clipSource;
        public BehaviorActionsRecord actions;
        public BehaviorTriggersRecord triggers;
        public BehaviorTimingsRecord timings;
    }
}
