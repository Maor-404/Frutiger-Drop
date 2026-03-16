using System;
using System.Runtime.InteropServices;

namespace FrutigerDrop;

public sealed class FrutigerDrop
{
    [StructLayout(LayoutKind.Sequential)]
    private struct FrutigerDropBuffer
    {
        public IntPtr ptr;
        public UIntPtr len;
    }

    // On Windows: frutiger_drop_core.dll
    // On Linux:   libfrutiger_drop_core.so
    // On macOS:   libfrutiger_drop_core.dylib
    private const string NativeLib = "frutiger_drop_core";

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "frutiger_drop_apply_blur")]
    private static extern FrutigerDropBuffer ApplyBlurNative(
        byte[] input,
        UIntPtr inputLen,
        uint width,
        uint height);

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "frutiger_drop_apply_tint")]
    private static extern FrutigerDropBuffer ApplyTintNative(
        byte[] rgba,
        UIntPtr rgbaLen,
        byte tr,
        byte tg,
        byte tb,
        byte ta);

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "frutiger_drop_composite_layers")]
    private static extern FrutigerDropBuffer CompositeLayersNative(
        byte[] bottom,
        UIntPtr bottomLen,
        byte[] top,
        UIntPtr topLen);

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "frutiger_drop_free")]
    private static extern void FreeNative(FrutigerDropBuffer buf);

    public byte[] ApplyBlur(byte[] inputRgba, uint width, uint height)
    {
        if (inputRgba is null) throw new ArgumentNullException(nameof(inputRgba));
        var buf = ApplyBlurNative(inputRgba, (UIntPtr)inputRgba.Length, width, height);
        return CopyAndFree(buf);
    }

    public byte[] ApplyTint(byte[] rgba, (byte r, byte g, byte b, byte a) tint)
    {
        if (rgba is null) throw new ArgumentNullException(nameof(rgba));
        var buf = ApplyTintNative(rgba, (UIntPtr)rgba.Length, tint.r, tint.g, tint.b, tint.a);
        return CopyAndFree(buf);
    }

    public byte[] CompositeLayers(byte[] bottomRgba, byte[] topRgba)
    {
        if (bottomRgba is null) throw new ArgumentNullException(nameof(bottomRgba));
        if (topRgba is null) throw new ArgumentNullException(nameof(topRgba));
        if (bottomRgba.Length != topRgba.Length) throw new ArgumentException("Layer buffers must match in length.");

        var buf = CompositeLayersNative(
            bottomRgba, (UIntPtr)bottomRgba.Length,
            topRgba, (UIntPtr)topRgba.Length);
        return CopyAndFree(buf);
    }

    private static byte[] CopyAndFree(FrutigerDropBuffer buf)
    {
        try
        {
            if (buf.ptr == IntPtr.Zero || buf.len == UIntPtr.Zero) return Array.Empty<byte>();
            checked
            {
                int len = (int)buf.len;
                var managed = new byte[len];
                Marshal.Copy(buf.ptr, managed, 0, len);
                return managed;
            }
        }
        finally
        {
            FreeNative(buf);
        }
    }
}

