using System.Text;
using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Controls.Templates;

namespace SVNexus.Components;

public abstract class LoadingOrErrorState {

    public static LoadingOrErrorState MakeLoading()
    {
        return new Loading()
        {
            LoadingMessage = "Loading",
        };
    }

    public static LoadingOrErrorState MakeError(string error = "")
    {
        return new Error()
        {
            ErrorMessage = error,
        };
    }

    public static LoadingOrErrorState MakeNone()
    {
        return new None();
    }
    
    public class Loading: LoadingOrErrorState
    {
        public object? LoadingMessage { get; set; }
        public IDataTemplate? LoadingMessageTemplate { get; set; }

        public virtual void SetLoadingMessage(LoadingOrError control)
        {
            control.LoadingMessage = LoadingMessage;
        }

        public virtual void SetLoadingMessageTemplate(LoadingOrError control)
        {
            control.LoadingMessageTemplate = LoadingMessageTemplate;
        }
        
    }
    public class Error: LoadingOrErrorState
    {
        public ICommand? RetryCommand { get; set; }
        public object? RetryCommandParameter { get; set; }
        public string? ErrorMessage { get; set; }

        public virtual void SetRetryCommand(LoadingOrError control)
        {
            control.RetryCommand = RetryCommand;
        }

        public virtual void SetRetryCommandParameter(LoadingOrError control)
        {
            control.RetryCommandParameter = RetryCommandParameter;
        }

        public virtual void SetErrorMessage(LoadingOrError control)
        {
            control.ErrorMessage = ErrorMessage;
        }
    }
    public class None: LoadingOrErrorState
    {
        
    }
}

public class LoadingOrError : ContentControl
{

    static LoadingOrError()
    {
        StateProperty.Changed.AddClassHandler<LoadingOrError>(OnStatePropertyChanged);
    }

    public static void OnStatePropertyChanged(LoadingOrError sender, AvaloniaPropertyChangedEventArgs e)
    {
        var isError = false;
        var isLoading = false;
        switch (e.NewValue)
        {
            case LoadingOrErrorState.Error error:
                error.SetErrorMessage(sender);
                error.SetRetryCommand(sender);
                error.SetRetryCommandParameter(sender);
                isError = true;
                break;
            case LoadingOrErrorState.Loading loading:
                loading.SetLoadingMessage(sender);
                loading.SetLoadingMessageTemplate(sender);
                isLoading = true;
                break;
        }
        
        sender.IsLoading = isLoading;
        sender.IsError = isError;
    }
    
    public static readonly StyledProperty<LoadingOrErrorState> StateProperty = AvaloniaProperty.Register<LoadingOrError, LoadingOrErrorState>(
        nameof(State), defaultValue: new LoadingOrErrorState.None());

    public LoadingOrErrorState State
    {
        get => GetValue(StateProperty);
        set => SetValue(StateProperty, value);
    }
    
    public static readonly StyledProperty<object?> RetryCommandParameterProperty = AvaloniaProperty.Register<LoadingOrError , object?>(
    nameof(RetryCommandParameter));

    public object? RetryCommandParameter
    {
        get => GetValue(RetryCommandParameterProperty);
        set => SetValue(RetryCommandParameterProperty, value);
    }

    public static readonly StyledProperty<ICommand?> RetryCommandProperty = AvaloniaProperty.Register<LoadingOrError , ICommand?>(
        nameof(RetryCommand));

    public ICommand? RetryCommand
    {
        get => GetValue(RetryCommandProperty);
        set => SetValue(RetryCommandProperty, value);
    }

    public static readonly StyledProperty<string?> ErrorMessageProperty = AvaloniaProperty.Register<LoadingOrError , string?>(
        nameof(ErrorMessage));

    public string? ErrorMessage
    {
        get => GetValue(ErrorMessageProperty);
        set => SetValue(ErrorMessageProperty, value);
    }

    public static readonly StyledProperty<bool> IsErrorProperty = AvaloniaProperty.Register<LoadingOrError , bool>(
        nameof(IsError));

    public bool IsError
    {
        get => GetValue(IsErrorProperty);
        set => SetValue(IsErrorProperty, value);
    }

    public static readonly StyledProperty<bool> IsLoadingProperty = AvaloniaProperty.Register<LoadingOrError , bool>(
        nameof(IsLoading));

    public bool IsLoading
    {
        get => GetValue(IsLoadingProperty);
        set => SetValue(IsLoadingProperty, value);
    }
    
    public static readonly StyledProperty<object?> LoadingMessageProperty = AvaloniaProperty.Register<LoadingOrError , object?>(
        nameof(LoadingMessage));

    public object? LoadingMessage
    {
        get => GetValue(LoadingMessageProperty);
        set => SetValue(LoadingMessageProperty, value);
    }

    public static readonly StyledProperty<IDataTemplate?> LoadingMessageTemplateProperty = AvaloniaProperty.Register<LoadingOrError , IDataTemplate?>(
        nameof(LoadingMessageTemplate));

    public IDataTemplate? LoadingMessageTemplate
    {
        get => GetValue(LoadingMessageTemplateProperty);
        set => SetValue(LoadingMessageTemplateProperty, value);
    }
}