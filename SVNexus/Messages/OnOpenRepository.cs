using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnOpenRepository(string value) : ValueChangedMessage<string>(value);