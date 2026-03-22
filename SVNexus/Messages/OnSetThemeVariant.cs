using Avalonia.Styling;
using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnSetThemeVariant(ThemeVariant? value) : ValueChangedMessage<ThemeVariant?>(value);