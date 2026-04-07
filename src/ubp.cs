// Unicode Path Bridge (UBP)
// A zero-dependency Windows utility that forwards command-line arguments
// with full Unicode fidelity to a target program, without flashing a console window.
//
// Compile with:
//   csc.exe /target:winexe /win32manifest:src\ubp.manifest /out:ubp.exe src\ubp.cs

using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;

class UnicodePathBridge
{
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern int MessageBoxW(IntPtr hWnd, string text, string caption, uint type);

    const uint MB_OK = 0x00000000;
    const uint MB_ICONERROR = 0x00000010;

    static int Main(string[] args)
    {
        if (args.Length < 1)
        {
            ShowError("Usage: ubp.exe <program> [arguments...]\n\nNo target program specified.");
            return 1;
        }

        string targetApp = args[0];
        string resolvedPath = ResolveExecutable(targetApp);

        if (resolvedPath == null)
        {
            ShowError("Target program not found: " + targetApp);
            return 2;
        }

        // Build the argument string for the child process.
        // Skip args[0] (the target program) and properly escape each remaining argument.
        string childArgs = string.Join(" ", args.Skip(1).Select(EscapeArgument));

        try
        {
            ProcessStartInfo psi = new ProcessStartInfo();
            psi.FileName = resolvedPath;
            psi.Arguments = childArgs;
            psi.UseShellExecute = false;
            psi.CreateNoWindow = true;
            psi.WindowStyle = ProcessWindowStyle.Hidden;

            Process.Start(psi);
            return 0;
        }
        catch (Exception ex)
        {
            ShowError("Failed to start: " + resolvedPath + "\n\n" + ex.Message);
            return 3;
        }
    }

    /// <summary>
    /// Escapes a single argument for the Windows command line, following the
    /// Microsoft C/C++ parameter parsing rules:
    /// https://learn.microsoft.com/en-us/cpp/c-language/parsing-c-command-line-arguments
    ///
    /// Rules:
    /// - Arguments containing spaces, tabs, or double quotes must be wrapped in double quotes.
    /// - Inside a quoted argument, backslashes are literal UNLESS they precede a double quote.
    /// - A sequence of N backslashes before a double quote becomes 2N+1 backslashes + a literal quote.
    /// - A sequence of N backslashes NOT before a double quote stays as N backslashes.
    /// - Empty arguments become "".
    /// </summary>
    static string EscapeArgument(string arg)
    {
        if (arg == null) return "\"\"";
        if (arg.Length == 0) return "\"\"";

        // If the argument has no special characters, return it as-is.
        bool needsQuoting = false;
        foreach (char c in arg)
        {
            if (c == ' ' || c == '\t' || c == '"')
            {
                needsQuoting = true;
                break;
            }
        }

        if (!needsQuoting) return arg;

        var sb = new StringBuilder(arg.Length + 4);
        sb.Append('"');

        int backslashCount = 0;
        foreach (char c in arg)
        {
            if (c == '\\')
            {
                // Count consecutive backslashes; we'll resolve them when we
                // hit a quote or the end of the string.
                backslashCount++;
            }
            else if (c == '"')
            {
                // Double the backslashes before the quote, then add an escaped quote.
                sb.Append('\\', backslashCount * 2 + 1);
                sb.Append('"');
                backslashCount = 0;
            }
            else
            {
                // Backslashes not before a quote are literal.
                sb.Append('\\', backslashCount);
                sb.Append(c);
                backslashCount = 0;
            }
        }

        // If the argument ends with backslashes, they'll sit right before the
        // closing quote — so we must double them.
        sb.Append('\\', backslashCount * 2);
        sb.Append('"');

        return sb.ToString();
    }

    /// <summary>
    /// Resolves a program name to a full path.
    /// - If it's already an absolute path and exists, return it.
    /// - Otherwise, search the system PATH.
    /// </summary>
    static string ResolveExecutable(string name)
    {
        // If an absolute or relative path was given, check it directly.
        if (name.IndexOf(Path.DirectorySeparatorChar) >= 0 ||
            name.IndexOf(Path.AltDirectorySeparatorChar) >= 0)
        {
            return File.Exists(name) ? Path.GetFullPath(name) : null;
        }

        // If the name already has an extension and exists in current dir, use it.
        if (Path.HasExtension(name) && File.Exists(name))
        {
            return Path.GetFullPath(name);
        }

        // Search PATH directories.
        string[] pathExts;
        if (Path.HasExtension(name))
        {
            pathExts = new string[] { "" };
        }
        else
        {
            // Try common executable extensions when none is specified.
            string pathExtEnv = Environment.GetEnvironmentVariable("PATHEXT");
            pathExts = pathExtEnv != null
                ? pathExtEnv.Split(';')
                : new string[] { ".exe", ".cmd", ".bat", ".com" };
        }

        string pathEnv = Environment.GetEnvironmentVariable("PATH");
        if (pathEnv == null) return null;

        string[] dirs = pathEnv.Split(';');
        foreach (string dir in dirs)
        {
            if (string.IsNullOrEmpty(dir)) continue;
            foreach (string ext in pathExts)
            {
                string candidate = Path.Combine(dir, name + ext);
                if (File.Exists(candidate))
                {
                    return candidate;
                }
            }
        }

        return null;
    }

    static void ShowError(string message)
    {
        MessageBoxW(IntPtr.Zero, message, "Unicode Path Bridge", MB_OK | MB_ICONERROR);
    }
}
