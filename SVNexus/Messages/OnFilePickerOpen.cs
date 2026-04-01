using System.Collections.Generic;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnFilePickerOpen: AsyncRequestMessage<IReadOnlyList<IStorageFile>>
{
    public required FilePickerOpenOptions Options { get; set; }
}